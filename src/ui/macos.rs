use std::fs::create_dir_all;

const NOTIFY_HELPER_BINARY : &'static [u8] = include_bytes!("res/mac/EspansoNotifyHelper.zip");

pub struct MacUIManager {

}

impl super::UIManager for MacUIManager {
    fn initialize(&self) {
        self.initialize_notify_helper();
    }

    fn notify(&self, message: &str) {
        unimplemented!()
    }
}

impl MacUIManager {
    fn initialize_notify_helper(&self) {
        let res = dirs::data_dir();
        if let Some(data_dir) = res {
            let espanso_dir = data_dir.join("espanso");

            let res = create_dir_all(espanso_dir);

            if let Ok(_) = res {
                // TODO: extract zip file

            }
        }

        // TODO: print error message
    }
}