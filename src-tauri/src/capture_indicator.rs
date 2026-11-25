use ashpd::zbus::{Connection, Proxy, Variant, proxy::SignalStream};
use futures_util::StreamExt;
use std::collections::HashMap;

pub struct CaptureIndicator {
    id: u32,
    proxy: Proxy<'static>,
    actions: SignalStream<'static>,
    closed: SignalStream<'static>,
}

impl CaptureIndicator {
    pub async fn show(connection: &Connection, source: &str) -> Result<Self, ashpd::zbus::Error> {
        let proxy = Proxy::new(
            connection,
            "org.freedesktop.Notifications",
            "/org/freedesktop/Notifications",
            "org.freedesktop.Notifications",
        )
        .await?;
        let capabilities: Vec<String> = proxy.call("GetCapabilities", &()).await?;
        if !["actions", "persistence"]
            .iter()
            .all(|required| capabilities.iter().any(|available| available == required))
        {
            return Err(ashpd::zbus::Error::Failure(
                "Desktop input controls unavailable".into(),
            ));
        }
        let actions = proxy.receive_signal("ActionInvoked").await?;
        let closed = proxy.receive_signal("NotificationClosed").await?;
        let hints = HashMap::from([
            ("resident", Variant::from(true)),
            ("suppress-sound", Variant::from(true)),
        ]);
        let id = proxy
            .call(
                "Notify",
                &(
                    "ShadowMind",
                    0_u32,
                    "",
                    source,
                    "Input remains local. You can stop it here.",
                    &["stop-input", "Stop input"][..],
                    hints,
                    0_i32,
                ),
            )
            .await?;
        Ok(Self {
            id,
            proxy,
            actions,
            closed,
        })
    }

    pub async fn stop_requested(&mut self) {
        loop {
            tokio::select! {
                signal = self.actions.next() => {
                    let Some(signal) = signal else { return };
                    if signal.body().deserialize::<(u32, &str)>().is_ok_and(|(id, action)| {
                        id == self.id && action == "stop-input"
                    }) {
                        return;
                    }
                }
                signal = self.closed.next() => {
                    let Some(signal) = signal else { return };
                    if signal.body().deserialize::<(u32, u32)>().is_ok_and(|(id, _)| id == self.id) {
                        return;
                    }
                }
            }
        }
    }

    pub async fn close(self) {
        let _ = self
            .proxy
            .call::<_, _, ()>("CloseNotification", &(self.id,))
            .await;
    }
}
