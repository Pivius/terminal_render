
pub fn rgba_to_grayscale(buffer: &[u8], width: usize, height: usize) -> Vec<u8> {
    // Create an output buffer for the grayscale image
    let mut grayscale_buffer = vec![0u8; width * height];

    // Check if the buffer length matches the expected size
    if buffer.len() != width * height * 4 {
        eprintln!("Error: Buffer size does not match the specified dimensions.");
        return grayscale_buffer; // Return empty buffer in case of error
    }

    // Iterate over each pixel in the buffer
    for y in 0..height {
        for x in 0..width {
            // Calculate the index for the RGBA values
            let index = (y * width + x) * 4;
            let r = buffer[index] as f32;
            let g = buffer[index + 1] as f32;
            let b = buffer[index + 2] as f32;

            // Convert RGB to grayscale using luminance formula
            let grayscale = (0.299 * r + 0.587 * g + 0.114 * b) as u8;

            // Set the grayscale value in the output buffer
            grayscale_buffer[y * width + x] = grayscale;
        }
    }

    grayscale_buffer
}

pub fn raw_buffer_to_ascii(raw_buffer: &mut [u8], width: usize, height: usize) {
    // Define ASCII characters for different intensity levels
    let ascii_chars = "@%#*+=-:. ";
    let raw_buffer = rgba_to_grayscale(raw_buffer, width, height);
    
    // Check if the buffer length matches the expected size
    if raw_buffer.len() != width * height {
        eprintln!("Error: Buffer size does not match the specified dimensions.");
        return;
    }

    // Create an output buffer for the ASCII representation
    let mut ascii_art = String::new();

    // Iterate over each pixel in the buffer
    for y in 0..height {
        for x in 0..width {
            let pixel_value = raw_buffer[y * width + x];
            
            // Map pixel value to an ASCII character
            let intensity_index = (pixel_value as usize * (ascii_chars.len() - 1)) / 255;
            let ascii_char = ascii_chars.chars().nth(intensity_index).unwrap();

            // Append the ASCII character to the output buffer
            ascii_art.push(ascii_char);
        }
        // Add a newline character after each row
        ascii_art.push('\n');
    }

    // Print the ASCII art to the command line
    println!("{}", ascii_art);
}
