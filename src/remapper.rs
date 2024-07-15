use crate::mapping::*;
use crate::state_machine::StateMachine;
use anyhow::*;
use evdev_rs::{Device, DeviceWrapper, GrabMode, ReadFlag, UInputDevice};
use std::path::Path;

pub struct InputMapper {
    input: Device,
    output: UInputDevice,

    state_machine: StateMachine,
}

fn enable_key_code(input: &mut Device, key: KeyCode) -> Result<()> {
    input
        .enable(EventCode::EV_KEY(key.clone()))
        .context(format!("enable key {:?}", key))?;
    Ok(())
}

impl InputMapper {
    pub fn create_mapper<P: AsRef<Path>>(path: P, mappings: &[Mapping]) -> Result<Self> {
        let path = path.as_ref();
        let f = std::fs::File::open(path).context(format!("opening {}", path.display()))?;
        let mut input = Device::new_from_file(f)
            .with_context(|| format!("failed to create new Device from file {}", path.display()))?;

        input.set_name(&format!("evremap Virtual input for {}", path.display()));

        // Ensure that any remapped keys are supported by the generated output device
        for map in mappings {
            match map {
                Mapping::Remap { mappings, .. } => {
                    for (_, o) in mappings.as_ref() {
                        for x in o.as_ref() {
                            enable_key_code(&mut input, *x)?;
                        }
                    }
                }
            }
        }

        let output = UInputDevice::create_from_device(&input)
            .context(format!("creating UInputDevice from {}", path.display()))?;

        input
            .grab(GrabMode::Grab)
            .context(format!("grabbing exclusive access on {}", path.display()))?;

        Ok(Self {
            input,
            output,
            state_machine: StateMachine::new(mappings),
        })
    }

    pub fn run_mapper(&mut self) -> Result<()> {
        log::info!("Going into read loop");
        loop {
            let (status, event) = self
                .input
                .next_event(ReadFlag::NORMAL | ReadFlag::BLOCKING)?;
            match status {
                evdev_rs::ReadStatus::Success => {
                    for e in self.state_machine.send(&event) {
                        self.output.write_event(&e)?;
                    }
                }
                evdev_rs::ReadStatus::Sync => bail!("ReadStatus::Sync!"),
            }
        }
    }
}
