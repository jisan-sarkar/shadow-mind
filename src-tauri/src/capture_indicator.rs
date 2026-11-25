use ashpd::zbus::{Connection, Proxy, Variant, proxy::SignalStream};
use futures_util::StreamExt;
use std::collections::HashMap;
use tokio::sync::OnceCell;

// GNOME can group notifications by application. Keep their sender alive when
// a single source closes its dedicated portal connection.
static NOTIFICATION_BUS: OnceCell<Connection> = OnceCell::const_new();

pub struct CaptureIndicator {
    id: u32,
    proxy: Proxy<'static>,
    actions: SignalStream<'static>,
    closed: SignalStream<'static>,
}

impl CaptureIndicator {
    pub async fn show(source: &str) -> Result<Self, ashpd::zbus::Error> {
        let connection = NOTIFICATION_BUS
            .get_or_try_init(Connection::session)
            .await?;
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
        // GNOME suppresses resident banners while their application has focus.
        // Keep this notification in the list until it is acted on or removed.
        let hints = HashMap::from([("suppress-sound", Variant::from(true))]);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::timeout;

    #[tokio::test]
    #[ignore = "run under dbus-run-session to isolate notification signals"]
    async fn only_the_matching_stop_action_ends_capture() {
        const PATH: &str = "/org/freedesktop/Notifications";
        const INTERFACE: &str = "org.freedesktop.Notifications";
        let service = Connection::session().await.unwrap();
        let client = Connection::session().await.unwrap();
        let proxy = Proxy::new(
            &client,
            service.unique_name().unwrap().clone(),
            PATH,
            INTERFACE,
        )
        .await
        .unwrap();
        let mut indicator = CaptureIndicator {
            id: 42,
            actions: proxy.receive_signal("ActionInvoked").await.unwrap(),
            closed: proxy.receive_signal("NotificationClosed").await.unwrap(),
            proxy,
        };
        let mut stop = tokio::spawn(async move { indicator.stop_requested().await });
        for action in [(41_u32, "stop-input"), (42_u32, "default")] {
            service
                .emit_signal(None::<()>, PATH, INTERFACE, "ActionInvoked", &action)
                .await
                .unwrap();
        }
        assert!(
            timeout(Duration::from_millis(100), &mut stop)
                .await
                .is_err()
        );
        service
            .emit_signal(
                None::<()>,
                PATH,
                INTERFACE,
                "ActionInvoked",
                &(42_u32, "stop-input"),
            )
            .await
            .unwrap();
        timeout(Duration::from_secs(2), stop)
            .await
            .unwrap()
            .unwrap();
    }
}
