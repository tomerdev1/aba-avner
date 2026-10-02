use image::{imageops::FilterType, DynamicImage};

pub fn compute_dhash(image: &DynamicImage) -> u64 {
    let resized = image.resize_exact(9, 8, FilterType::Triangle).to_luma8();
    let mut hash: u64 = 0;

    for y in 0..8 {
        for x in 0..8 {
            let left = resized.get_pixel(x, y)[0];
            let right = resized.get_pixel(x + 1, y)[0];
            let bit_index = (y * 8 + x) as u64;
            if left > right {
                hash |= 1u64 << bit_index;
            }
        }
    }

    hash
}

pub fn hamming_distance(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};

    fn solid_color(color: [u8; 3]) -> DynamicImage {
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(32, 32, |_x, _y| Rgb(color));
        DynamicImage::ImageRgb8(img)
    }

    fn striped_columns() -> DynamicImage {
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(9, 8, |x, _y| {
            if x % 2 == 0 {
                Rgb([250, 250, 250])
            } else {
                Rgb([5, 5, 5])
            }
        });
        DynamicImage::ImageRgb8(img)
    }

    #[test]
    fn dhash_identical_images_match() {
        let img = solid_color([10, 20, 30]);
        let h1 = compute_dhash(&img);
        let h2 = compute_dhash(&img);
        assert_eq!(h1, h2);
        assert_eq!(hamming_distance(h1, h2), 0);
    }

    #[test]
    fn dhash_detects_difference() {
        let img_a = solid_color([10, 20, 30]);
        let img_b = striped_columns();
        let h1 = compute_dhash(&img_a);
        let h2 = compute_dhash(&img_b);
        assert!(hamming_distance(h1, h2) > 0);
    }
}
