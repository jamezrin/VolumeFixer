use std::fmt::Debug;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

#[allow(unused_imports)]
use windows::{
    core::*,
    Win32::*,
    Win32::Foundation::*,
    Win32::Media::Audio::*,
    Win32::Media::Audio::Endpoints::*,
    Win32::System::*,
    Win32::System::Com::*,
    Win32::Devices::FunctionDiscovery::*,
    Win32::UI::*,
    Win32::UI::Shell::*,
};

use log::{info, warn};
use windows::Win32::UI::Shell::PropertiesSystem::{PropVariantToBSTR, PropVariantToString, PropVariantToStringAlloc, PSCreatePropertyStoreFromObject};

#[implement(IAudioEndpointVolumeCallback)]
struct AppIAudioEndpointVolumeCallback {}

#[allow(non_snake_case)]
impl IAudioEndpointVolumeCallback_Impl for AppIAudioEndpointVolumeCallback {
    fn OnNotify(&self, pnotify: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> Result<()> {
        let change = unsafe { pnotify.as_ref().unwrap() };

        info!(
            "OnNotify {:?} {} {} {} {}",
            change.guidEventContext,
            change.fMasterVolume,
            change.nChannels,
            change.bMuted.0,
            change.afChannelVolumes[0]
        );

        Ok(())
    }
}

#[implement(IMMNotificationClient)]
struct AppIMMNotificationClient {}

#[allow(non_snake_case)]
impl IMMNotificationClient_Impl for AppIMMNotificationClient {
    fn OnDeviceStateChanged(&self, pwstrdeviceid: &PCWSTR, dwnewstate: u32) -> Result<()> {
        info!(
            "OnDeviceStateChanged {} {}",
            unsafe { pwstrdeviceid.to_string()? },
            dwnewstate
        );

        Ok(())
    }

    fn OnDeviceAdded(&self, pwstrdeviceid: &PCWSTR) -> Result<()> {
        info!(
            "OnDeviceAdded {}",
            unsafe { pwstrdeviceid.to_string()? }
        );

        Ok(())
    }

    fn OnDeviceRemoved(&self, pwstrdeviceid: &PCWSTR) -> Result<()> {
        info!(
            "OnDeviceRemoved {}",
            unsafe { pwstrdeviceid.to_string()? }
        );

        Ok(())
    }

    fn OnDefaultDeviceChanged(&self, flow: EDataFlow, role: ERole, pwstrdefaultdeviceid: &PCWSTR) -> Result<()> {
        info!(
            "OnDefaultDeviceChanged {} {} {}",
            flow.0,
            role.0,
            unsafe { pwstrdefaultdeviceid.to_string()? }
        );

        Ok(())
    }

    fn OnPropertyValueChanged(&self, pwstrdeviceid: &PCWSTR, key: &PropertiesSystem::PROPERTYKEY) -> Result<()> {
        info!(
            "OnPropertyValueChanged {} {:?}",
            unsafe { pwstrdeviceid.to_string()? },
            key.fmtid
        );

        Ok(())
    }
}

fn hook_all_audio_devices() -> Result<()> {
    unsafe {
        // mandatory initialization of COM subsystem
        CoInitializeEx(None, COINIT_MULTITHREADED)?;

        // get instance to IMMDeviceEnumerator interface
        let device_enumerator: IMMDeviceEnumerator = CoCreateInstance(
            &MMDeviceEnumerator,
            None,
            CLSCTX_INPROC_SERVER
        )?;

        // register changes in audio endpoint devices
        let endpoint_notification_cb = AppIMMNotificationClient {};
        let interface : IMMNotificationClient = endpoint_notification_cb.into();
        device_enumerator.RegisterEndpointNotificationCallback(&interface)?;

        let device = device_enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;

        hook_audio_device(&device)?;
    }

    Ok(())
}

fn hook_audio_device(device: &IMMDevice) -> Result<()> {
    unsafe {
        // https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/nf-mmdeviceapi-immdevice-activate
        let audio_endpoint_volume = device.Activate::<IAudioEndpointVolume>(
            CLSCTX_ALL,
            None
        )?;

        // https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Media/Audio/Endpoints/struct.IAudioEndpointVolumeCallback.html
        let volume_control_cb = AppIAudioEndpointVolumeCallback {};
        let interface : IAudioEndpointVolumeCallback = volume_control_cb.into();
        audio_endpoint_volume.RegisterControlChangeNotify(&interface)?;

        let property_store = device.OpenPropertyStore(STGM_READ)?;

        // https://github.com/microsoft/windows-rs/issues/1685
        // https://github.com/microsoft/windows-rs/issues/595
        /*
        pub struct PWSTR(pub *mut u16);

        impl PWSTR {
            /// Construct a new `PWSTR` from a raw pointer.
            pub const fn from_raw(ptr: *mut u16) -> Self {
                Self(ptr)
            }

            /// Construct a null `PWSTR`.
            pub fn null() -> Self {
                Self(std::ptr::null_mut())
            }

            /// Returns a raw pointer to the `PWSTR`.
            pub fn as_ptr(&self) -> *mut u16 {
                self.0
            }
        */
        let friendly_name_ret = PropVariantToStringAlloc(
            &property_store.GetValue(&PKEY_Device_FriendlyName)?
        )?;

        let friendly_name = friendly_name_ret.to_string()?;

        CoTaskMemFree(Some(friendly_name_ret.as_ptr() as *const core::ffi::c_void));

        info!("RegisterControlChangeNotify {:?} {:?} {}", friendly_name, device, device.GetId().unwrap().to_string()?);

        /*
        loop {
            // println!("audio {}", audio_endpoint_volume.GetMasterVolumeLevelScalar()?);
            sleep(Duration::from_millis(10));
        }
         */

        // https://rust-cli.github.io/book/in-depth/signals.html

        /*
        let device_coll = device_enumerator.EnumAudioEndpoints(
            eAll,
            DEVICE_STATE_ACTIVE | DEVICE_STATE_UNPLUGGED
        )?;

        for ndevice in 0..device_coll.GetCount()? {
            let device = device_coll.Item(ndevice)?;
            let property_store = device.OpenPropertyStore(STGM_READ)?;

            println!("ndevice {} {:?}: ", ndevice, device)

        }
        */

        // very important, scope must be maintained
        loop {}
    }

    Ok(())
}

fn main() {
    env_logger::init();

    thread::spawn(move || {
        hook_all_audio_devices()
            .expect("Fail to hook audio devices");
    });

    loop {
        sleep(Duration::from_millis(10));
    }
}
