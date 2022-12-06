use crate::result::CipherErr;
use crate::testingtools::{pipe, MutexExt, Spy, Tx};
use crate::use_cases::cipher::{
    Cipher, CipherRead, CipherReadStrategy, CipherStrategy, CipherWrite, CipherWriteStrategy,
};

use anyhow::Result;
use std::sync::Arc;
use tracing::debug;

pub fn tracked(cipher: &Cipher) -> (CipherSpies, Cipher) {
    TrackedCipher::wrap(cipher)
}

pub struct TrackedCipher {
    read: CipherRead,
    write: CipherWrite,
}

impl TrackedCipher {
    fn wrap(cipher: &Cipher) -> (CipherSpies, Cipher) {
        let (read_tx, read_spy) = pipe();
        let (write_tx, write_spy) = pipe();

        (
            CipherSpies::new(read_spy, write_spy),
            Box::new(Self {
                read: TrackedCipherRead::create(cipher.read(), read_tx),
                write: TrackedCipherWrite::create(cipher.write(), write_tx),
            }),
        )
    }
}

impl CipherStrategy for TrackedCipher {
    fn read(&self) -> CipherRead {
        self.read.clone()
    }

    fn write(&self) -> CipherWrite {
        self.write.clone()
    }
}

pub struct TrackedCipherRead {
    read: CipherRead,
    #[allow(unused)]
    tx: Tx,
}

impl TrackedCipherRead {
    fn create(read: CipherRead, tx: Tx) -> CipherRead {
        Arc::new(Self { read, tx })
    }
}

impl CipherReadStrategy for TrackedCipherRead {
    fn decrypt(&self, src_buf: &[u8]) -> Result<Vec<u8>, CipherErr> {
        self.read.decrypt(src_buf)
    }
}

pub struct TrackedCipherWrite {
    write: CipherWrite,
    tx: Tx,
}

impl TrackedCipherWrite {
    fn create(write: CipherWrite, tx: Tx) -> CipherWrite {
        Arc::new(Self { write, tx })
    }
}

impl CipherWriteStrategy for TrackedCipherWrite {
    fn encrypt(&self, src_buf: &[u8]) -> Result<Vec<u8>, CipherErr> {
        debug!("before encrypting");
        self.tx.signal();
        let res = self.write.encrypt(src_buf)?;
        debug!("after encryption");
        Ok(res)
    }
}

pub struct CipherSpies {
    #[allow(unused)]
    read_spy: Spy,
    write_spy: Spy,
}

impl CipherSpies {
    fn new(read_spy: Spy, write_spy: Spy) -> Self {
        Self {
            read_spy,
            write_spy,
        }
    }

    #[allow(unused)]
    pub fn decrypt_called(&self) -> bool {
        self.read_spy.method_called()
    }

    pub fn encrypt_called(&self) -> bool {
        self.write_spy.method_called()
    }
}

pub fn failing() -> Cipher {
    FailingCipher::make()
}

pub struct FailingCipher {
    read: CipherRead,
    write: CipherWrite,
}

impl FailingCipher {
    fn make() -> Cipher {
        Box::new(Self {
            read: FailingCipherRead::new(),
            write: FailingCipherWrite::new(),
        })
    }
}

impl CipherStrategy for FailingCipher {
    fn read(&self) -> CipherRead {
        self.read.clone()
    }

    fn write(&self) -> CipherWrite {
        self.write.clone()
    }
}

pub struct FailingCipherRead;

impl FailingCipherRead {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl CipherReadStrategy for FailingCipherRead {
    fn decrypt(&self, _src_buf: &[u8]) -> Result<Vec<u8>, CipherErr> {
        Err(CipherErr::Chacha(chacha20poly1305::Error))
    }
}

pub struct FailingCipherWrite;

impl FailingCipherWrite {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl CipherWriteStrategy for FailingCipherWrite {
    fn encrypt(&self, _src_buf: &[u8]) -> Result<Vec<u8>, CipherErr> {
        Err(CipherErr::Chacha(chacha20poly1305::Error))
    }
}

pub fn working() -> Cipher {
    WorkingCipher::make()
}

struct WorkingCipher {
    read: CipherRead,
    write: CipherWrite,
}

impl WorkingCipher {
    fn make() -> Cipher {
        Box::new(Self {
            read: WorkingCipherRead::new(),
            write: WorkingCipherWrite::new(),
        })
    }
}

impl CipherStrategy for WorkingCipher {
    fn read(&self) -> CipherRead {
        self.read.clone()
    }

    fn write(&self) -> CipherWrite {
        self.write.clone()
    }
}

struct WorkingCipherRead;

impl WorkingCipherRead {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl CipherReadStrategy for WorkingCipherRead {
    fn decrypt(&self, _buf: &[u8]) -> Result<Vec<u8>, CipherErr> {
        Ok(Vec::new())
    }
}

struct WorkingCipherWrite;

impl WorkingCipherWrite {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl CipherWriteStrategy for WorkingCipherWrite {
    fn encrypt(&self, _src_buf: &[u8]) -> std::result::Result<Vec<u8>, CipherErr> {
        Ok(Vec::new())
    }
}

pub fn noop() -> Cipher {
    NoOpCipher::make()
}

struct NoOpCipher {
    read: CipherRead,
    write: CipherWrite,
}

impl NoOpCipher {
    fn make() -> Cipher {
        Box::new(Self {
            read: NoOpCipherRead::new(),
            write: NoOpCipherWrite::new(),
        })
    }
}

impl CipherStrategy for NoOpCipher {
    fn read(&self) -> CipherRead {
        self.read.clone()
    }

    fn write(&self) -> CipherWrite {
        self.write.clone()
    }
}

struct NoOpCipherRead;

impl NoOpCipherRead {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl CipherReadStrategy for NoOpCipherRead {
    fn decrypt(&self, _buf: &[u8]) -> Result<Vec<u8>, CipherErr> {
        // nothing to do
        Ok(Vec::new())
    }
}

struct NoOpCipherWrite;

impl NoOpCipherWrite {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl CipherWriteStrategy for NoOpCipherWrite {
    fn encrypt(&self, _src_buf: &[u8]) -> std::result::Result<Vec<u8>, CipherErr> {
        // nothing to do
        Ok(Vec::new())
    }
}
