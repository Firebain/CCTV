use xml::writer::EventWriter;

pub trait WriterOwner {
    fn borrow_writer(self) -> EventWriter<Vec<u8>>;
    
    fn get_writer(&mut self) -> &mut EventWriter<Vec<u8>>;
}