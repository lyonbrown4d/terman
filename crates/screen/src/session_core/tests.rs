use super::{ScreenControlEvent, ScreenSessionBus, ScreenSessionEvent};

#[test]
fn replays_recent_output_to_attach_clients() {
    let bus = ScreenSessionBus::new();
    bus.publish_output(b"hello");

    assert_eq!(bus.replay_snapshot(), b"hello".to_vec());
}

#[test]
fn subscribes_with_replay_without_losing_snapshot() {
    let bus = ScreenSessionBus::new();
    bus.publish_output(b"hello");
    let (replay, subscription) = bus.subscribe_with_replay(None);
    bus.publish_output(b"!");

    assert_eq!(replay, b"hello".to_vec());
    assert_eq!(
        subscription.try_recv(),
        Ok(ScreenSessionEvent::Output(b"!".to_vec()))
    );
}

#[test]
fn tracks_attach_client_count_for_replay_subscriptions() {
    let bus = ScreenSessionBus::new();
    let (_replay, subscription) = bus.subscribe_with_replay(None);

    assert_eq!(bus.status_snapshot().attach_clients, 1);
    drop(subscription);
    assert_eq!(bus.status_snapshot().attach_clients, 0);
}

#[test]
fn detaches_one_client_without_broadcasting() {
    let bus = ScreenSessionBus::new();
    let (_replay, subscription) = bus.subscribe_with_replay(Some(String::from("client")));

    bus.detach_client("client");

    assert!(subscription.recv().is_err());
    drop(subscription);
    assert_eq!(bus.status_snapshot().attach_clients, 0);
}

#[test]
fn tracks_last_message() {
    let bus = ScreenSessionBus::new();
    let (_replay, subscription) = bus.subscribe_with_replay(None);
    bus.publish_message(b"message\r\n");

    assert_eq!(bus.last_message_snapshot(), b"message\r\n".to_vec());
    assert_eq!(
        subscription.try_recv(),
        Ok(ScreenSessionEvent::Output(b"message\r\n".to_vec()))
    );
}
#[test]
fn updates_scrollback_limit() {
    let bus = ScreenSessionBus::new();
    bus.set_scrollback_lines(0);
    bus.publish_output(b"hello");

    assert_eq!(bus.status_snapshot().scrollback_lines, 0);
    assert!(bus.replay_snapshot().is_empty());
}

#[test]
fn updates_window_title() {
    let bus = ScreenSessionBus::new();
    bus.set_window_title(String::from("editor"));

    assert_eq!(bus.status_snapshot().window_title.as_deref(), Some("editor"));
}

#[test]
fn renumbers_window_and_swaps_existing_index() {
    let bus = ScreenSessionBus::new();
    bus.add_window(1, Some(String::from("editor")));
    bus.renumber_window(1, 0);
    let status = bus.status_snapshot();
    let active = status.windows.iter().find(|window| window.active).unwrap();

    assert_eq!(status.active_window, 0);
    assert_eq!(active.index, 0);
    assert_eq!(active.title.as_deref(), Some("editor"));
    assert!(status.windows.iter().any(|window| window.index == 1 && !window.active));
}
#[test]
fn models_attach_control_events() {
    assert_eq!(
        ScreenControlEvent::Input(b"x".to_vec()),
        ScreenControlEvent::Input(b"x".to_vec())
    );
}