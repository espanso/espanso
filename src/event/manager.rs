use crate::event::{KeyEventReceiver, ActionEventReceiver, Event};
use std::sync::mpsc::Receiver;

pub trait EventManager {
    fn eventloop(&self);
}

pub struct DefaultEventManager<'a, K: KeyEventReceiver, A: ActionEventReceiver> {
    receive_channel: Receiver<Event>,
    key_receivers: Vec<&'a K>,
    action_receivers: Vec<&'a A>,
}

impl<'a, K: KeyEventReceiver, A: ActionEventReceiver> DefaultEventManager<'a, K, A> {
    pub fn new(receive_channel: Receiver<Event>, key_receivers: Vec<&'a K>,
               action_receivers: Vec<&'a A>) -> DefaultEventManager<'a, K, A> {
        DefaultEventManager {
            receive_channel,
            key_receivers,
            action_receivers,
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
                            self.key_receivers.iter().for_each(move |&receiver| receiver.on_key_event(key_event.clone()));
                        },
                        Event::Action(action_event) => {
                            self.action_receivers.iter().for_each(|&receiver| receiver.on_action_event(action_event.clone()));
                        }
                    }
                },
                Err(_) => panic!("Broken event channel"),
            }
        }
    }
}