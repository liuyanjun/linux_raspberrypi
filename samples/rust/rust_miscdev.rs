// SPDX-License-Identifier: GPL-2.0

//!
//! How to build only modules:
//! make LLVM=-17 O=build_4b ARCH=arm64 M=samples/rust
//!
//! How to use in qemu:
//! / # sudo insmod rust_miscdev.ko
//! / # sudo cat /proc/misc  -> c 10 122
//! / # sudo chmod 777 /dev/rust_misc
//! / # sudo echo "hello" > /dev/rust_misc
//! / # sudo cat /dev/rust_misc  -> Hello
//! 

use kernel::prelude::*;
use kernel::{
    file::{self, File},
    io_buffer::{IoBufferReader, IoBufferWriter},
    sync::{Arc, ArcBorrow},
    sync::Mutex,
    miscdev, 
    pin_init,
    new_mutex,
    fmt,
};

use core::ops::{Deref, DerefMut};
use core::result::Result::Ok;

module! {
    type: RustMiscDev,
    name: "rust_miscdev",
    author: "i dont konw",
    description: "Rust exercise 002",
    license: "GPL",
}

const GLOBALMEM_SIZE: usize = 0x1000;

#[pin_data]
struct RustMiscdevData {
    #[pin]
    inner: Mutex<[u8;GLOBALMEM_SIZE]>,
}

impl RustMiscdevData {
    fn try_new() -> Result<Arc<Self>>{
        pr_info!("rust miscdevice created\n");
        Ok(Arc::pin_init(
            pin_init!(Self {
                inner <- new_mutex!([0u8;GLOBALMEM_SIZE])
            })
        )?)
    }
}

unsafe impl Sync for RustMiscdevData {}
unsafe impl Send for RustMiscdevData {}

// unit struct for file operations
struct RustFile;

#[vtable]
impl file::Operations for RustFile {
    type Data = Arc<RustMiscdevData>;
    type OpenData = Arc<RustMiscdevData>;

    fn open(shared: &Arc<RustMiscdevData>, _file: &file::File) -> Result<Self::Data> {
        pr_info!("open in miscdevice\n",);
        //TODO
        return Ok(shared.clone())
    }

    fn read(
        shared: ArcBorrow<'_, RustMiscdevData>,
        _file: &File,
        writer: &mut impl IoBufferWriter,
        offset: u64,
    ) -> Result<usize> {
        pr_info!("read in miscdevice offset:{}, len:{}\n", offset, writer.len());
        //TODO
        if offset >= GLOBALMEM_SIZE as u64{
            return Ok(0)
        }

        let holder = shared.deref().inner.lock();

        let len = core::cmp::min(offset + writer.len() as u64, GLOBALMEM_SIZE as u64);

        let _ = writer.write_slice(&holder[offset as usize .. len as usize])?;
        Ok(len as usize)
        
    }

    fn write(
        shared: ArcBorrow<'_, RustMiscdevData>,
        _file: &File,
        reader: &mut impl IoBufferReader,
        offset: u64,
    ) -> Result<usize> {
        pr_info!("write in miscdevice offset:{}, len:{}\n", offset, reader.len());
        //TODO
        if offset >= GLOBALMEM_SIZE as u64{
            return Ok(0)
        }

        let mut holder = shared.deref().inner.lock();
        let data: &mut [u8;GLOBALMEM_SIZE] = holder.deref_mut();

        let len = core::cmp::min(offset + reader.len() as u64, GLOBALMEM_SIZE as u64);

        reader.read_slice(&mut data[offset as usize..len as usize])?;
         
        Ok(len as usize)

    }

    fn release(_data: Self::Data, _file: &File) {
        pr_info!("release in miscdevice\n");
    }
}

struct RustMiscDev {
    _dev: Pin<Box<miscdev::Registration<RustFile>>>,
}

impl kernel::Module for RustMiscDev {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust miscdevice device sample (init)\n");

        let data: Arc<RustMiscdevData> = RustMiscdevData::try_new()?;

        let misc_reg = miscdev::Registration::new_pinned(fmt!("rust_misc"), data)?;

        Ok(RustMiscDev { _dev: misc_reg })
    }
}

impl Drop for RustMiscDev {
    fn drop(&mut self) {
        pr_info!("Rust miscdevice device sample (exit)\n");
    }
}