use crate::result::CipherErr;
use crate::testingtools::{pipe, MutexExt, Spy, Tx};
use crate::use_cases::cipher::{
    Cipher, CipherReader, CipherReaderStrategy, CipherStrategy, CipherWriter, CipherWriterStrategy,
};

use anyhow::Result;
use std::sync::Arc;
use tracing::debug;

pub fn tracked(cipher: &Cipher) -> (CipherSpies, Cipher) {
    TrackedCipher::wrap(cipher)
}

pub struct TrackedCipher {
    reader: CipherReader,
    writer: CipherWriter,
}

impl TrackedCipher {
    fn wrap(cipher: &Cipher) -> (CipherSpies, Cipher) {
        let (read_tx, read_spy) = pipe();
        let (write_tx, write_spy) = pipe();

        (
            CipherSpies::new(read_spy, write_spy),
            Box::new(Self {
                reader: TrackedCipherRead::create(cipher.reader(), read_tx),
                writer: TrackedCipherWrite::create(cipher.writer(), write_tx),
            }),
        )
    }
}

impl CipherStrategy for TrackedCipher {
    fn reader(&self) -> CipherReader {
        self.reader.clone()
    }

    fn writer(&self) -> CipherWriter {
        self.writer.clone()
    }
}

pub struct TrackedCipherRead {
    reader: CipherReader,
    #[allow(unused)]
    tx: Tx,
}

impl TrackedCipherRead {
    fn create(reader: CipherReader, tx: Tx) -> CipherReader {
        Arc::new(Self { reader, tx })
    }
}

impl CipherReaderStrategy for TrackedCipherRead {
    fn decrypt(&self, src_buf: &[u8]) -> Result<Vec<u8>, CipherErr> {
        self.reader.decrypt(src_buf)
    }
}

pub struct TrackedCipherWrite {
    write: CipherWriter,
    tx: Tx,
}

impl TrackedCipherWrite {
    fn create(write: CipherWriter, tx: Tx) -> CipherWriter {
        Arc::new(Self { write, tx })
    }
}

impl CipherWriterStrategy for TrackedCipherWrite {
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
    reader_spy: Spy,
    writer_spy: Spy,
}

impl CipherSpies {
    fn new(reader_spy: Spy, writer_spy: Spy) -> Self {
        Self {
            reader_spy,
            writer_spy,
        }
    }

    #[allow(unused)]
    pub fn decrypt_called(&self) -> bool {
        self.reader_spy.method_called()
    }

    pub fn encrypt_called(&self) -> bool {
        self.writer_spy.method_called()
    }
}

pub fn failing() -> Cipher {
    FailingCipher::make()
}

pub struct FailingCipher {
    reader: CipherReader,
    writer: CipherWriter,
}

impl FailingCipher {
    fn make() -> Cipher {
        Box::new(Self {
            reader: FailingCipherReader::new(),
            writer: FailingCipherWriter::new(),
        })
    }
}

impl CipherStrategy for FailingCipher {
    fn reader(&self) -> CipherReader {
        self.reader.clone()
    }

    fn writer(&self) -> CipherWriter {
        self.writer.clone()
    }
}

pub struct FailingCipherReader;

impl FailingCipherReader {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl CipherReaderStrategy for FailingCipherReader {
    fn decrypt(&self, _src_buf: &[u8]) -> Result<Vec<u8>, CipherErr> {
        Err(CipherErr::Chacha(chacha20poly1305::Error))
    }
}

pub struct FailingCipherWriter;

impl FailingCipherWriter {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl CipherWriterStrategy for FailingCipherWriter {
    fn encrypt(&self, _src_buf: &[u8]) -> Result<Vec<u8>, CipherErr> {
        Err(CipherErr::Chacha(chacha20poly1305::Error))
    }
}

pub fn working() -> Cipher {
    WorkingCipher::make()
}

struct WorkingCipher {
    reader: CipherReader,
    writer: CipherWriter,
}

impl WorkingCipher {
    fn make() -> Cipher {
        Box::new(Self {
            reader: WorkingCipherReader::new(),
            writer: WorkingCipherWriter::new(),
        })
    }
}

impl CipherStrategy for WorkingCipher {
    fn reader(&self) -> CipherReader {
        self.reader.clone()
    }

    fn writer(&self) -> CipherWriter {
        self.writer.clone()
    }
}

struct WorkingCipherReader;

impl WorkingCipherReader {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl CipherReaderStrategy for WorkingCipherReader {
    fn decrypt(&self, _buf: &[u8]) -> Result<Vec<u8>, CipherErr> {
        Ok(Vec::new())
    }
}

struct WorkingCipherWriter;

impl WorkingCipherWriter {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl CipherWriterStrategy for WorkingCipherWriter {
    fn encrypt(&self, _src_buf: &[u8]) -> std::result::Result<Vec<u8>, CipherErr> {
        Ok(Vec::new())
    }
}

pub fn noop() -> Cipher {
    NoOpCipher::make()
}

struct NoOpCipher {
    reader: CipherReader,
    writer: CipherWriter,
}

impl NoOpCipher {
    fn make() -> Cipher {
        Box::new(Self {
            reader: NoOpCipherReader::new(),
            writer: NoOpCipherWriter::new(),
        })
    }
}

impl CipherStrategy for NoOpCipher {
    fn reader(&self) -> CipherReader {
        self.reader.clone()
    }

    fn writer(&self) -> CipherWriter {
        self.writer.clone()
    }
}

struct NoOpCipherReader;

impl NoOpCipherReader {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl CipherReaderStrategy for NoOpCipherReader {
    fn decrypt(&self, _buf: &[u8]) -> Result<Vec<u8>, CipherErr> {
        // nothing to do
        Ok(Vec::new())
    }
}

struct NoOpCipherWriter;

impl NoOpCipherWriter {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl CipherWriterStrategy for NoOpCipherWriter {
    fn encrypt(&self, _src_buf: &[u8]) -> std::result::Result<Vec<u8>, CipherErr> {
        // nothing to do
        Ok(Vec::new())
    }
}
