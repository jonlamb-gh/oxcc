// https://github.com/jonlamb-gh/oscc/tree/devel/firmware/can_gateway

use fault_can_protocol::*;
use nucleo_f767zi::hal::can::{CanError, CanFrame, DataFrame, RxFifo};
use nucleo_f767zi::hal::prelude::*;
use oscc_magic_byte::*;
use types::*;
use vehicle::*;

// TODO - use some form of println! logging that prefixes with a module name?

pub struct CanGatewayModule {
    can_publish_timer: CanPublishTimer,
    control_can: ControlCan,
    obd_can: ObdCan,
    fault_report_can_frame: DataFrame,
}

impl CanGatewayModule {
    pub fn new(
        can_publish_timer: CanPublishTimer,
        control_can: ControlCan,
        obd_can: ObdCan,
    ) -> Self {
        CanGatewayModule {
            can_publish_timer,
            control_can,
            obd_can,
            fault_report_can_frame: default_fault_report_data_frame(),
        }
    }
    pub fn republish_obd_frames_to_control_can_bus(&mut self) {
        // poll both OBD CAN FIFOs
        for fifo in &[RxFifo::Fifo0, RxFifo::Fifo1] {
            if let Ok(rx_frame) = self.obd_can().receive(fifo) {
                self.republish_obd_frame_to_control_can_bus(&rx_frame);
            }
        }
    }

    // TODO - error handling
    fn republish_obd_frame_to_control_can_bus(&mut self, frame: &CanFrame) {
        let id: u32 = frame.id().into();
        let mut is_a_match = false;

        if (id == KIA_SOUL_OBD_STEERING_WHEEL_ANGLE_CAN_ID.into())
            || (id == KIA_SOUL_OBD_WHEEL_SPEED_CAN_ID.into())
            || (id == KIA_SOUL_OBD_BRAKE_PRESSURE_CAN_ID.into())
        {
            is_a_match = true;
        }

        #[cfg(feature = "kia-soul-ev")]
        {
            if id == KIA_SOUL_OBD_THROTTLE_PRESSURE_CAN_ID.into() {
                is_a_match = true;
            }
        }

        if is_a_match && self.control_can().transmit(&frame).is_err() {
            // TODO - error handling
        }
    }

    // TODO - hide these details, switch to a publisher approach
    pub fn control_can(&mut self) -> &mut ControlCan {
        &mut self.control_can
    }

    pub fn obd_can(&mut self) -> &mut ObdCan {
        &mut self.obd_can
    }

    pub fn wait_for_publish(&mut self) -> bool {
        self.can_publish_timer.wait().is_ok()
    }
}

impl FaultReportPublisher for CanGatewayModule {
    fn publish_fault_report(&mut self, fault_report: &OsccFaultReport) -> Result<(), CanError> {
        {
            self.fault_report_can_frame
                .set_data_length(OSCC_FAULT_REPORT_CAN_DLC as _);

            let data = self.fault_report_can_frame.data_as_mut();

            data[0] = OSCC_MAGIC_BYTE_0;
            data[1] = OSCC_MAGIC_BYTE_1;
            data[2] = (fault_report.fault_origin_id & 0xFF) as _;
            data[3] = ((fault_report.fault_origin_id >> 8) & 0xFF) as _;
            data[4] = ((fault_report.fault_origin_id >> 16) & 0xFF) as _;
            data[5] = ((fault_report.fault_origin_id >> 24) & 0xFF) as _;
            data[6] = fault_report.dtcs;
        }

        self.control_can
            .transmit(&self.fault_report_can_frame.into())
    }
}
