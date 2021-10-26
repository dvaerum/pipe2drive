use std::io;
use std::io::Read;

pub struct TestBuffer {
    counter: usize,
    size: usize,
}

impl TestBuffer {
    pub fn new(size: usize) -> TestBuffer {
        TestBuffer {
            counter: 0,
            size: size,
        }
    }
}

impl Read for TestBuffer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut new_counter: usize = self.counter + buf.len();
        if new_counter > self.size {
            new_counter = self.size;
        }

        let mut tmp_counter: usize = 0;
        for current_count in self.counter..new_counter {
            let i = current_count - self.counter;
            // 48 is the DEC value for the CHAR '0',
            // so it is used to convert to str/char
            buf[i] = (current_count % 10 + 48) as u8;

            tmp_counter += 1;
        }
        self.counter = new_counter;

        return Ok(tmp_counter);
    }
}
