use std::{fs, io::Write};

use elderscroll::{
    dbi::DbiStream,
    directory::{Stream, DBI_STREAM_INDEX, INVALID_STREAM_SIZE},
    msf::BigMsf,
    omap::{OmapEntry, OmapStream},
    pagelist::PageList,
    view::SourceView,
};

/// This test just moves 2 functions to padding inbetween
/// Existing functions. Just a demo to show that we can move code.
#[test]
fn omap_test1() {
    let bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/bins/HelloWorld.pdb"
    ));
    let mut msf = BigMsf::new(bytes.to_vec());
    let page_size = msf.header().unwrap().get_page_size();
    let mut stream_directory = msf.get_stream_directory().unwrap();
    let dbi_stream = stream_directory.streams[DBI_STREAM_INDEX].clone();
    assert!(dbi_stream.original_stream_size != INVALID_STREAM_SIZE);
    let mut dbi = DbiStream::new(dbi_stream);
    dbi.nop_section_maps().unwrap();

    // Set original section headers the same as the "section headers" stream.
    let mut extras = dbi.extra_streams_mut().unwrap();
    extras.set_original_section_headers(extras.get_section_headers());

    // Set the omap to src.
    let omap_stream_index = stream_directory.streams.len();
    extras.set_omap_to_src(omap_stream_index as u16);
    let mut omap_stream = OmapStream::default();
    omap_stream.0.insert(OmapEntry(0x1008, 0x1000));
    omap_stream.0.insert(OmapEntry(0x100f, 0x1007));
    omap_stream.0.insert(OmapEntry(0x1010, 0x1010));
    omap_stream.0.insert(OmapEntry(0x1088, 0x1010));
    omap_stream.0.insert(OmapEntry(0x109F, 0x1064));
    omap_stream.0.insert(OmapEntry(0x10A0, 0x10A0));

    let omap_bytes = omap_stream.to_vec().unwrap();

    // Omap to src.
    stream_directory.streams.push(Stream {
        original_stream_size: Default::default(),
        view: SourceView {
            bytes: omap_bytes,
            pages: PageList::new(page_size),
        },
    });

    // Omap from src
    let omap_stream_index2 = stream_directory.streams.len();
    extras.set_omap_from_src(omap_stream_index2 as u16);
    let mut omap_stream2 = OmapStream::default();
    omap_stream2.0.insert(OmapEntry(0x1000, 0x1008));
    omap_stream2.0.insert(OmapEntry(0x1007, 0x100f));
    omap_stream2.0.insert(OmapEntry(0x1010, 0x1088));
    omap_stream2.0.insert(OmapEntry(0x1064, 0x109F));
    omap_stream2.0.insert(OmapEntry(0x10A0, 0x10A0));
    let omap_bytes2 = omap_stream2.to_vec().unwrap();
    eprintln!("bytes: {:X?}", omap_bytes2);
    stream_directory.streams.push(Stream {
        original_stream_size: Default::default(),
        view: SourceView {
            bytes: omap_bytes2,
            pages: PageList::new(page_size),
        },
    });

    stream_directory.streams[DBI_STREAM_INDEX] = dbi.stream;
    msf.set_stream_directory(stream_directory).unwrap();
    let header = msf.header().unwrap();

    assert_eq!(
        header.get_num_pages() * header.get_page_size(),
        msf.bytes.len() as u32
    );

    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/bins/HelloWorld_new.pdb"
        ))
        .unwrap();

    file.write_all(&msf.bytes).unwrap();
}
