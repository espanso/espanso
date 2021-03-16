// Clip Library
// Copyright (c) 2018 David Capello
//
// This file is released under the terms of the MIT license.
// Read LICENSE.txt for more information.

#include "clip.h"

#include <algorithm>
#include <vector>

#include "png.h"

namespace clip {
namespace x11 {

//////////////////////////////////////////////////////////////////////
// Functions to convert clip::image into png data to store it in the
// clipboard.

void write_data_fn(png_structp png, png_bytep buf, png_size_t len) {
  std::vector<uint8_t>& output = *(std::vector<uint8_t>*)png_get_io_ptr(png);
  const size_t i = output.size();
  output.resize(i+len);
  std::copy(buf, buf+len, output.begin()+i);
}

bool write_png(const image& image,
               std::vector<uint8_t>& output) {
  png_structp png = png_create_write_struct(PNG_LIBPNG_VER_STRING,
                                            nullptr, nullptr, nullptr);
  if (!png)
    return false;

  png_infop info = png_create_info_struct(png);
  if (!info) {
    png_destroy_write_struct(&png, nullptr);
    return false;
  }

  if (setjmp(png_jmpbuf(png))) {
    png_destroy_write_struct(&png, &info);
    return false;
  }

  png_set_write_fn(png,
                   (png_voidp)&output,
                   write_data_fn,
                   nullptr);    // No need for a flush function

  const image_spec& spec = image.spec();
  int color_type = (spec.alpha_mask ?
                    PNG_COLOR_TYPE_RGB_ALPHA:
                    PNG_COLOR_TYPE_RGB);

  png_set_IHDR(png, info,
               spec.width, spec.height, 8, color_type,
               PNG_INTERLACE_NONE, PNG_COMPRESSION_TYPE_BASE, PNG_FILTER_TYPE_BASE);
  png_write_info(png, info);
  png_set_packing(png);

  png_bytep row =
    (png_bytep)png_malloc(png, png_get_rowbytes(png, info));

  for (png_uint_32 y=0; y<spec.height; ++y) {
    const uint32_t* src =
      (const uint32_t*)(((const uint8_t*)image.data())
                        + y*spec.bytes_per_row);
    uint8_t* dst = row;
    unsigned int x, c;

    for (x=0; x<spec.width; x++) {
      c = *(src++);
      *(dst++) = (c & spec.red_mask  ) >> spec.red_shift;
      *(dst++) = (c & spec.green_mask) >> spec.green_shift;
      *(dst++) = (c & spec.blue_mask ) >> spec.blue_shift;
      if (color_type == PNG_COLOR_TYPE_RGB_ALPHA)
        *(dst++) = (c & spec.alpha_mask) >> spec.alpha_shift;
    }

    png_write_rows(png, &row, 1);
  }

  png_free(png, row);
  png_write_end(png, info);
  png_destroy_write_struct(&png, &info);
  return true;
}

//////////////////////////////////////////////////////////////////////
// Functions to convert png data stored in the clipboard to a
// clip::image.

struct read_png_io {
  const uint8_t* buf;
  size_t len;
  size_t pos;
};

void read_data_fn(png_structp png, png_bytep buf, png_size_t len) {
  read_png_io& io = *(read_png_io*)png_get_io_ptr(png);
  if (io.pos < io.len) {
    size_t n = std::min(len, io.len-io.pos);
    if (n > 0) {
      std::copy(io.buf+io.pos,
                io.buf+io.pos+n,
                buf);
      io.pos += n;
    }
  }
}

bool read_png(const uint8_t* buf,
              const size_t len,
              image* output_image,
              image_spec* output_spec) {
  png_structp png = png_create_read_struct(PNG_LIBPNG_VER_STRING,
                                           nullptr, nullptr, nullptr);
  if (!png)
    return false;

  png_infop info = png_create_info_struct(png);
  if (!info) {
    png_destroy_read_struct(&png, nullptr, nullptr);
    return false;
  }

  if (setjmp(png_jmpbuf(png))) {
    png_destroy_read_struct(&png, &info, nullptr);
    return false;
  }

  read_png_io io = { buf, len, 0 };
  png_set_read_fn(png, (png_voidp)&io, read_data_fn);

  png_read_info(png, info);

  png_uint_32 width, height;
  int bit_depth, color_type, interlace_type;
  png_get_IHDR(png, info, &width, &height,
               &bit_depth, &color_type,
               &interlace_type,
               nullptr, nullptr);

  image_spec spec;
  spec.width = width;
  spec.height = height;
  spec.bits_per_pixel = 32;
  spec.bytes_per_row = png_get_rowbytes(png, info);

  spec.red_mask    = 0x000000ff;
  spec.green_mask  = 0x0000ff00;
  spec.blue_mask   = 0x00ff0000;
  spec.red_shift   = 0;
  spec.green_shift = 8;
  spec.blue_shift  = 16;

  // TODO indexed images with alpha
  if (color_type == PNG_COLOR_TYPE_RGB_ALPHA ||
      color_type == PNG_COLOR_TYPE_GRAY_ALPHA) {
    spec.alpha_mask = 0xff000000;
    spec.alpha_shift = 24;
  }
  else {
    spec.alpha_mask = 0;
    spec.alpha_shift = 0;
  }

  if (output_spec)
    *output_spec = spec;

  if (output_image &&
      width > 0 &&
      height > 0) {
    image img(spec);

    // We want RGB 24-bit or RGBA 32-bit as a result
    png_set_strip_16(png); // Down to 8-bit (TODO we might support 16-bit values)
    png_set_packing(png);  // Use one byte if color depth < 8-bit
    png_set_expand_gray_1_2_4_to_8(png);
    png_set_palette_to_rgb(png);
    png_set_gray_to_rgb(png);
    png_set_tRNS_to_alpha(png);

    int number_passes = png_set_interlace_handling(png);
    png_read_update_info(png, info);

    png_bytepp rows = (png_bytepp)png_malloc(png, sizeof(png_bytep)*height);
    png_uint_32 y;
    for (y=0; y<height; ++y)
      rows[y] = (png_bytep)png_malloc(png, spec.bytes_per_row);

    for (int pass=0; pass<number_passes; ++pass)
      for (y=0; y<height; ++y)
        png_read_rows(png, rows+y, nullptr, 1);

    for (y=0; y<height; ++y) {
      const uint8_t* src = rows[y];
      uint32_t* dst = (uint32_t*)(img.data() + y*spec.bytes_per_row);
      unsigned int x, r, g, b, a = 255;

      for (x=0; x<width; x++) {
        r = *(src++);
        g = *(src++);
        b = *(src++);
        if (spec.alpha_mask)
          a = *(src++);
        *(dst++) =
          (r << spec.red_shift) |
          (g << spec.green_shift) |
          (b << spec.blue_shift) |
          (a << spec.alpha_shift);
      }
      png_free(png, rows[y]);
    }
    png_free(png, rows);

    std::swap(*output_image, img);
  }

  png_destroy_read_struct(&png, &info, nullptr);
  return true;
}

} // namespace x11
} // namespace clip
