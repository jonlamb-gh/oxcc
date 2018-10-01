use bootloader_can_protocol::*;
use nucleo_f767zi::hal::can::CanFrame;
use oxcc_bootloader_lib::reset_to_bootloader;
use oxcc_error::OxccError;

// TODO - cleanup CAN protocols and namespaces, use/validate data bytes?
pub fn process_rx_frame(can_frame: &CanFrame) -> Result<(), OxccError> {
    if let CanFrame::DataFrame(ref frame) = can_frame {
        let id: u32 = frame.id().into();
        let dlc = frame.data().len();

        if id == u32::from(OSCC_BOOTLOADER_RESET_CAN_ID) {
            if dlc == usize::from(OSCC_BOOTLOADER_RESET_CAN_DLC) {
                reset_to_bootloader();
            }
        }
    }

    Ok(())
}
