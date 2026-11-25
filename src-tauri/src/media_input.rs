use crate::capture_indicator::CaptureIndicator;
use ashpd::{
    desktop::{
        PersistMode, ResponseError, Session,
        screencast::{CursorMode, Screencast, SelectSourcesOptions, SourceType},
    },
    zbus::{Connection, Proxy},
};
use futures_util::{Stream, StreamExt};
use gstreamer::{self as gst, prelude::*};
use gstreamer_app::AppSink;
use serde::Serialize;
use std::{
    os::fd::{AsRawFd, OwnedFd},
    time::Duration,
};
use tokio::{sync::oneshot, time::timeout};
use uuid::Uuid;

pub const BUS_TIMEOUT: Duration = Duration::from_secs(5);
pub const PORTAL_TIMEOUT: Duration = Duration::from_secs(120);
const REVOCATION_GRACE: Duration = Duration::from_millis(350);

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NativeStatus {
    #[default]
    Idle,
    Connecting,
    Disconnecting,
    Receiving,
    Unavailable,
    Denied,
    Suspended,
    CleanupFailed,
    ControlsUnavailable,
}

#[derive(Clone, Debug, Serialize)]
pub struct AudioChoice {
    pub id: Uuid,
    pub label: String,
    #[serde(skip)]
    pub target: String,
}

pub enum MediaSource {
    Screen,
    Audio(String),
}

pub fn audio_choices() -> Result<Vec<AudioChoice>, NativeStatus> {
    gst::init().map_err(|_| NativeStatus::Unavailable)?;
    let monitor = gst::DeviceMonitor::new();
    monitor.add_filter(Some("Audio/Source"), None);
    monitor.start().map_err(|_| NativeStatus::Unavailable)?;
    let devices = monitor.devices();
    monitor.stop();
    let choices = devices
        .iter()
        .filter_map(|device| {
            let element = device.create_element(None).ok()?;
            if element.factory()?.name() != "pipewiresrc" {
                return None;
            }
            let target = element.property::<Option<String>>("target-object")?;
            let serial = device.properties()?.get::<String>("object.serial").ok()?;
            if target != serial || serial.parse::<u64>().is_err() {
                return None;
            }
            let label: String = device
                .display_name()
                .chars()
                .filter(|character| !character.is_control())
                .take(100)
                .collect();
            Some(AudioChoice {
                id: Uuid::new_v4(),
                label,
                target,
            })
        })
        .take(32)
        .collect();
    Ok(choices)
}

pub async fn receive(
    source: MediaSource,
    mut cancellation: oneshot::Receiver<()>,
    available: impl Fn(),
) -> NativeStatus {
    let setup = async {
        let desktop = Connection::session().await?;
        let system = Connection::system().await?;
        Ok::<_, ashpd::zbus::Error>((desktop, system))
    };
    let (desktop, system) = match tokio::select! {
        biased;
        _ = &mut cancellation => return NativeStatus::Idle,
        connections = timeout(BUS_TIMEOUT, setup) => connections,
    } {
        Ok(Ok(connections)) => connections,
        _ => return NativeStatus::Unavailable,
    };

    let mut capture = Capture::default();
    let mut indicator = None;
    let result = tokio::select! {
        biased;
        _ = &mut cancellation => NativeStatus::Idle,
        result = receive_unlocked(
            source, &desktop, &system, &mut capture, &mut indicator, available
        ) => result,
    };
    if capture.release(desktop, indicator).await {
        result
    } else {
        NativeStatus::CleanupFailed
    }
}

#[derive(Default)]
struct Capture {
    session: Option<Session<Screencast>>,
    pipeline: Option<Pipeline>,
    pipewire_remote: Option<OwnedFd>,
}

impl Capture {
    async fn release(self, desktop: Connection, indicator: Option<CaptureIndicator>) -> bool {
        let Self {
            session,
            pipeline,
            pipewire_remote,
        } = self;
        let release_pipeline = async move {
            let Some(pipeline) = pipeline else {
                return true;
            };
            let stop = tauri::async_runtime::spawn_blocking(move || {
                let _pipewire_remote = pipewire_remote;
                pipeline.stop()
            });
            matches!(timeout(BUS_TIMEOUT, stop).await, Ok(Ok(true)))
        };
        let release_portal = async move {
            let close_session = async {
                let Some(session) = session else { return };
                // Closed sessions may reject Close. Disconnecting the unique caller
                // also releases its pending requests and session.
                let _ = session.close().await;
            };
            let close_indicator = async {
                if let Some(indicator) = indicator {
                    indicator.close().await;
                }
            };
            let _ = timeout(Duration::from_secs(1), async {
                tokio::join!(close_session, close_indicator);
            })
            .await;
            matches!(timeout(BUS_TIMEOUT, desktop.close()).await, Ok(Ok(())))
        };
        let (pipeline_released, portal_released) = tokio::join!(release_pipeline, release_portal);
        pipeline_released && portal_released
    }
}

async fn receive_unlocked(
    source: MediaSource,
    desktop: &Connection,
    system: &Connection,
    resources: &mut Capture,
    indicator: &mut Option<CaptureIndicator>,
    available: impl Fn(),
) -> NativeStatus {
    let setup = lifecycle_streams(desktop, system);
    let Ok(Ok((mut locks, mut sleeps, locked))) = timeout(BUS_TIMEOUT, setup).await else {
        return NativeStatus::Unavailable;
    };
    if locked {
        return NativeStatus::Suspended;
    }
    let label = match &source {
        MediaSource::Screen => "ShadowMind screen input",
        MediaSource::Audio(_) => "ShadowMind audio input",
    };
    let Ok(Ok(control)) = timeout(BUS_TIMEOUT, CaptureIndicator::show(desktop, label)).await else {
        return NativeStatus::ControlsUnavailable;
    };
    let control = indicator.insert(control);
    let capture = async {
        if gst::init().is_err() {
            return NativeStatus::Unavailable;
        }
        match source {
            MediaSource::Screen => capture_screen(desktop, resources, available).await,
            MediaSource::Audio(target) => capture_audio(target, resources, available).await,
        }
    };
    tokio::select! {
        biased;
        _ = locks.next() => NativeStatus::Suspended,
        _ = sleeps.next() => NativeStatus::Suspended,
        _ = control.stop_requested() => NativeStatus::Idle,
        result = capture => result,
    }
}

async fn lifecycle_streams(
    desktop: &Connection,
    system: &Connection,
) -> Result<
    (
        ashpd::zbus::proxy::SignalStream<'static>,
        ashpd::zbus::proxy::SignalStream<'static>,
        bool,
    ),
    ashpd::zbus::Error,
> {
    let screen_saver = Proxy::new(
        desktop,
        "org.gnome.ScreenSaver",
        "/org/gnome/ScreenSaver",
        "org.gnome.ScreenSaver",
    )
    .await?;
    let login = Proxy::new(
        system,
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
    )
    .await?;
    let locks = screen_saver.receive_signal("ActiveChanged").await?;
    let sleeps = login.receive_signal("PrepareForSleep").await?;
    let locked = screen_saver.call::<_, _, bool>("GetActive", &()).await?;
    Ok((locks, sleeps, locked))
}

pub async fn watch_desktop(suspend: impl Fn(), resume: impl Fn()) {
    loop {
        let setup = async {
            let desktop = Connection::session().await?;
            let system = Connection::system().await?;
            let streams = lifecycle_streams(&desktop, &system).await?;
            Ok::<_, ashpd::zbus::Error>((desktop, system, streams))
        };
        if let Ok(Ok((_desktop, _system, (mut locks, mut sleeps, mut locked)))) =
            timeout(BUS_TIMEOUT, setup).await
        {
            let mut sleeping = false;
            if locked {
                suspend()
            } else {
                resume()
            }
            loop {
                let (from_lock, signal) = tokio::select! {
                    message = locks.next() => (true, message),
                    message = sleeps.next() => (false, message),
                };
                let Some(active) =
                    signal.and_then(|message| message.body().deserialize::<bool>().ok())
                else {
                    break;
                };
                if from_lock {
                    locked = active
                } else {
                    sleeping = active
                }
                if locked || sleeping {
                    suspend()
                } else {
                    resume()
                }
            }
        }
        suspend();
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn capture_screen(
    desktop: &Connection,
    resources: &mut Capture,
    available: impl Fn(),
) -> NativeStatus {
    let Ok(Ok(portal)) = timeout(BUS_TIMEOUT, Screencast::with_connection(desktop.clone())).await
    else {
        return NativeStatus::Unavailable;
    };
    let Ok(Ok(session)) = timeout(BUS_TIMEOUT, portal.create_session(Default::default())).await
    else {
        return NativeStatus::Unavailable;
    };
    let session = resources.session.insert(session);
    let setup = async {
        let Ok(revoked) = session.receive_closed().await else {
            return Err(NativeStatus::Unavailable);
        };
        let request = portal
            .select_sources(
                session,
                SelectSourcesOptions::default()
                    .set_sources(SourceType::WINDOW | SourceType::MONITOR)
                    .set_cursor_mode(CursorMode::HIDDEN)
                    .set_multiple(false)
                    .set_persist_mode(PersistMode::DoNot),
            )
            .await;
        if request.and_then(|request| request.response()).is_err() {
            return Err(NativeStatus::Unavailable);
        }
        let chosen = portal
            .start(session, None, Default::default())
            .await
            .and_then(|request| request.response());
        let selected = match chosen {
            Ok(selected) => selected,
            Err(ashpd::Error::Response(ResponseError::Cancelled)) => {
                return Err(NativeStatus::Denied);
            }
            Err(_) => return Err(NativeStatus::Unavailable),
        };
        let [stream] = selected.streams() else {
            return Err(NativeStatus::Unavailable);
        };
        if !matches!(
            stream.source_type(),
            Some(SourceType::WINDOW | SourceType::MONITOR)
        ) || selected.restore_token().is_some()
        {
            return Err(NativeStatus::Unavailable);
        }
        let Ok(fd) = portal
            .open_pipe_wire_remote(session, Default::default())
            .await
        else {
            return Err(NativeStatus::Unavailable);
        };
        let Some(source) = pipewire_source() else {
            return Err(NativeStatus::Unavailable);
        };
        source.set_property("fd", fd.as_raw_fd());
        source.set_property("path", stream.pipe_wire_node_id().to_string());
        let Some(pipeline) = Pipeline::screen(source) else {
            return Err(NativeStatus::Unavailable);
        };
        Ok((revoked, fd, pipeline))
    };
    let (revoked, fd, pipeline) = match timeout(PORTAL_TIMEOUT, setup).await {
        Ok(Ok(resources)) => resources,
        Ok(Err(status)) => return status,
        Err(_) => return NativeStatus::Unavailable,
    };
    resources.pipewire_remote = Some(fd);
    let pipeline = resources.pipeline.insert(pipeline);
    screen_outcome(revoked, pipeline.consume(available)).await
}

async fn screen_outcome(
    mut revoked: impl Stream + Unpin,
    receive: impl Future<Output = NativeStatus>,
) -> NativeStatus {
    tokio::select! {
        biased;
        closed = revoked.next() => {
            if closed.is_some() { NativeStatus::Denied } else { NativeStatus::Unavailable }
        }
        result = receive => {
            // The PipeWire error and the portal's Closed signal can arrive in either order.
            if timeout(REVOCATION_GRACE, revoked.next())
                .await.is_ok_and(|signal| signal.is_some())
            {
                NativeStatus::Denied
            } else {
                result
            }
        }
    }
}

async fn capture_audio(
    target: String,
    resources: &mut Capture,
    available: impl Fn(),
) -> NativeStatus {
    let Some(source) = pipewire_source() else {
        return NativeStatus::Unavailable;
    };
    source.set_property("target-object", target);
    let Some(pipeline) = Pipeline::audio(source) else {
        return NativeStatus::Unavailable;
    };
    resources.pipeline.insert(pipeline).consume(available).await
}

fn pipewire_source() -> Option<gst::Element> {
    let element = gst::ElementFactory::make("pipewiresrc").build().ok()?;
    if element.find_property("on-disconnect").is_some() {
        element.set_property_from_str("on-disconnect", "error");
    }
    element.set_property(
        "stream-properties",
        gst::Structure::builder("properties")
            .field("node.dont-fallback", true)
            .field("node.dont-reconnect", true)
            .build(),
    );
    Some(element)
}

struct Pipeline {
    gstreamer: gst::Pipeline,
    sink: AppSink,
    max_buffer: usize,
    video: bool,
}

impl Pipeline {
    fn stop(self) -> bool {
        self.gstreamer.set_state(gst::State::Null).is_ok()
            && matches!(
                self.gstreamer.state(gst::ClockTime::from_seconds(2)),
                (Ok(_), gst::State::Null, _)
            )
    }
    fn screen(source: gst::Element) -> Option<Self> {
        let caps = gst::Caps::builder("video/x-raw")
            .field("format", "RGBA")
            .field("width", gst::IntRange::new(1, 1920))
            .field("height", gst::IntRange::new(1, 1080))
            .field("framerate", gst::Fraction::new(5, 1))
            .build();
        let rate = gst::ElementFactory::make("videorate")
            .property("drop-only", true)
            .build()
            .ok()?;
        let transforms = vec![
            gst::ElementFactory::make("videoconvert").build().ok()?,
            gst::ElementFactory::make("videoscale").build().ok()?,
            rate,
        ];
        Self::new(source, transforms, caps, 1920 * 1080 * 4, true)
    }

    fn audio(source: gst::Element) -> Option<Self> {
        let caps = gst::Caps::builder("audio/x-raw")
            .field("format", "S16LE")
            .field("rate", 16_000_i32)
            .field("channels", 1_i32)
            .build();
        let transforms = vec![
            gst::ElementFactory::make("audioconvert").build().ok()?,
            gst::ElementFactory::make("audioresample").build().ok()?,
        ];
        Self::new(source, transforms, caps, 16_000 * 2, false)
    }

    fn new(
        source: gst::Element,
        transforms: Vec<gst::Element>,
        caps: gst::Caps,
        max_buffer: usize,
        video: bool,
    ) -> Option<Self> {
        let gstreamer = gst::Pipeline::new();
        let sink = AppSink::builder()
            .caps(&caps)
            .sync(false)
            .max_buffers(1)
            .drop(true)
            .build();
        let mut elements = vec![source];
        elements.extend(transforms);
        elements.push(sink.clone().upcast());
        gstreamer.add_many(&elements).ok()?;
        gst::Element::link_many(&elements).ok()?;
        Some(Self {
            gstreamer,
            sink,
            max_buffer,
            video,
        })
    }

    async fn consume(&self, available: impl Fn()) -> NativeStatus {
        if self.gstreamer.set_state(gst::State::Playing).is_err() {
            return NativeStatus::Unavailable;
        }
        let Some(bus) = self.gstreamer.bus() else {
            return NativeStatus::Unavailable;
        };
        let first_frame_deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        let mut received = false;
        loop {
            if bus
                .pop_filtered(&[gst::MessageType::Error, gst::MessageType::Eos])
                .is_some()
            {
                return NativeStatus::Unavailable;
            }
            if let Some(sample) = self.sink.try_pull_sample(gst::ClockTime::ZERO)
                && !received
                && sample.buffer().is_some_and(|buffer| self.usable(buffer))
            {
                received = true;
                available();
            }
            if !received && tokio::time::Instant::now() >= first_frame_deadline {
                return NativeStatus::Unavailable;
            }
            tokio::time::sleep(Duration::from_millis(35)).await;
        }
    }
    fn usable(&self, buffer: &gst::BufferRef) -> bool {
        (1..=self.max_buffer).contains(&buffer.size())
            && (!self.video
                || buffer.map_readable().is_ok_and(|mapped| {
                    mapped
                        .as_slice()
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .step_by(32)
                        .any(|pixel| pixel[0] > 5 || pixel[1] > 5 || pixel[2] > 5)
                }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[tokio::test]
    async fn screen_failure_requires_a_portal_signal_to_report_revocation() {
        use futures_util::stream;
        assert_eq!(
            screen_outcome(stream::iter([()]), std::future::pending()).await,
            NativeStatus::Denied,
        );
        let delayed_close =
            stream::once(async { tokio::time::sleep(Duration::from_millis(20)).await });
        assert_eq!(
            screen_outcome(Box::pin(delayed_close), async { NativeStatus::Unavailable }).await,
            NativeStatus::Denied,
        );
        assert_eq!(
            screen_outcome(stream::pending::<()>(), async { NativeStatus::Unavailable }).await,
            NativeStatus::Unavailable,
        );
    }

    #[test]
    fn synthetic_audio_has_a_bounded_sample_and_releases_pipeline() {
        gst::init().unwrap();
        let source = gst::ElementFactory::make("audiotestsrc")
            .property("is-live", true)
            .property("num-buffers", 30_i32)
            .build()
            .unwrap();
        let pipeline = Pipeline::audio(source).unwrap();
        let graph = pipeline.gstreamer.clone();
        let received = AtomicBool::new(false);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();
        let outcome = runtime.block_on(async {
            tokio::time::timeout(
                Duration::from_secs(4),
                pipeline.consume(|| received.store(true, Ordering::Relaxed)),
            )
            .await
        });
        assert_eq!(outcome.unwrap(), NativeStatus::Unavailable);
        assert!(received.load(Ordering::Relaxed));
        assert!(pipeline.stop());
        assert_eq!(graph.current_state(), gst::State::Null);
    }

    #[test]
    fn black_or_oversized_pixels_are_not_accepted_as_available() {
        gst::init().unwrap();
        let source = gst::ElementFactory::make("videotestsrc").build().unwrap();
        let pipeline = Pipeline::screen(source).unwrap();
        assert!(!pipeline.usable(&gst::Buffer::from_slice(vec![0_u8, 0, 0, 255])));
        assert!(pipeline.usable(&gst::Buffer::from_slice(vec![160_u8, 0, 0, 255])));
        assert!(!pipeline.usable(&gst::Buffer::with_size(pipeline.max_buffer + 1).unwrap()));
    }
}
