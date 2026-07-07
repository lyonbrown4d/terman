use super::{TmuxSessionBus, TmuxSessionEvent};

#[test]
fn replays_recent_output_to_attach_clients() {
    let bus = TmuxSessionBus::new(1);
    bus.publish_output(b"hello");
    assert_eq!(bus.replay_snapshot(), b"hello".to_vec());
}

#[test]
fn subscribes_with_replay_without_losing_snapshot() {
    let bus = TmuxSessionBus::new(1);
    bus.publish_output(b"hello");
    let (replay, subscription) = bus.subscribe_with_replay(None);
    bus.publish_output(b"!");
    assert_eq!(replay, b"hello".to_vec());
    assert_eq!(subscription.try_recv(), Ok(TmuxSessionEvent::Output(b"!".to_vec())));
}