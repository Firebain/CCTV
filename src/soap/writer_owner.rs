use xml::writer::EventWriter;
use std::io::Write;

pub trait WriterOwner<W: Write> {
    fn borrow_writer(self) -> EventWriter<W>;
    
    fn get_writer(&mut self) -> &mut EventWriter<W>;
}