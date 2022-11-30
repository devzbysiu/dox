use crate::result::CipherErr;
use crate::testingtools::{pipe, MutexExt, Spy, Tx};
use crate::use_cases::cipher::{
    Cipher, CipherRead, CipherReadStrategy, CipherStrategy, CipherWrite, CipherWriteStrategy,
};

use anyhow::Result;
use std::sync::Arc;
use tracing::debug;

pub struct TrackedCipher {
    read: CipherRead,
    write: CipherWrite,
}

impl TrackedCipher {
    pub fn wrap(cipher: &Cipher) -> (CipherSpies, Cipher) {
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
pub struct FailingCipher {
    read: CipherRead,
    write: CipherWrite,
}

impl FailingCipher {
    pub fn create() -> Cipher {
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
    pub fn read(&self) -> &Spy {
        &self.read_spy
    }

    pub fn write(&self) -> &Spy {
        &self.write_spy
    }
}
