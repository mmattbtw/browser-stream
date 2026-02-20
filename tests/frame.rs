use browser_stream::frame::decode_screencast_frame;

#[test]
fn decodes_and_resizes_frame() {
    // 1x1 red PNG
    let png_base64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAAAAAA6fptVAAAACklEQVR4nGNgAAAAAgABSK+kcQAAAABJRU5ErkJggg==";

    let frame = decode_screencast_frame(png_base64, 2, 2).expect("decode should work");

    assert_eq!(frame.width, 2);
    assert_eq!(frame.height, 2);
    assert_eq!(frame.data.len(), 2 * 2 * 3);
}
