use crate::event::{KeyEventReceiver, ActionEventReceiver, Event};
use std::sync::mpsc::Receiver;

pub trait EventManager {
    fn eventloop(&self);
}

pub struct DefaultEventManager<'a, K: KeyEventReceiver, A: ActionEventReceiver> {
    receive_channel: Receiver<Event>,
    key_receiver: &'a K,
    action_receiver: &'a A,
}

impl<'a, K: KeyEventReceiver, A: ActionEventReceiver> DefaultEventManager<'a, K, A> {
    pub fn new(receive_channel: Receiver<Event>, key_receiver: &'a K,
               action_receiver: &'a A) -> DefaultEventManager<'a, K, A> {
        DefaultEventManager {
            receive_channel,
            key_receiver,
            action_receiver,
        }
    }
}

impl <'a, K: KeyEventReceiver, A: ActionEventReceiver> EventManager for DefaultEventManager<'a, K, A> {
    fn eventloop(&self) {
        loop {
            match self.receive_channel.recv() {
                Ok(event) => {
                    match event {
                        Event::Key(key_event) => {
                            self.key_receiver.on_key_event(key_event);
                        },
                        Event::Action => {
                            self.action_receiver.on_action_event(event);  // TODO: action event
                        }
                    }
                },
                Err(_) => panic!("Broken event channel"),
            }
        }
    }
}