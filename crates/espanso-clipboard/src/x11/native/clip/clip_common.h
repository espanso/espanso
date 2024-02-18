// Clip Library
// Copyright (C) 2020 David Capello
//
// This file is released under the terms of the MIT license.
// Read LICENSE.txt for more information.

#ifndef CLIP_COMMON_H_INCLUDED
#define CLIP_COMMON_H_INCLUDED
#pragma once

namespace clip {
namespace details {

inline void divide_rgb_by_alpha(image& img,
                                bool hasAlphaGreaterThanZero = false) {
  const image_spec& spec = img.spec();

  bool hasValidPremultipliedAlpha = true;

  for (unsigned long y=0; y<spec.height; ++y) {
    const uint32_t* dst = (uint32_t*)(img.data()+y*spec.bytes_per_row);
    for (unsigned long x=0; x<spec.width; ++x, ++dst) {
      const uint32_t c = *dst;
      const int r = ((c & spec.red_mask  ) >> spec.red_shift  );
      const int g = ((c & spec.green_mask) >> spec.green_shift);
      const int b = ((c & spec.blue_mask ) >> spec.blue_shift );
      const int a = ((c & spec.alpha_mask) >> spec.alpha_shift);

      if (a > 0)
        hasAlphaGreaterThanZero = true;
      if (r > a || g > a || b > a)
        hasValidPremultipliedAlpha = false;
    }
  }

  for (unsigned long y=0; y<spec.height; ++y) {
    uint32_t* dst = (uint32_t*)(img.data()+y*spec.bytes_per_row);
    for (unsigned long x=0; x<spec.width; ++x, ++dst) {
      const uint32_t c = *dst;
      int r = ((c & spec.red_mask  ) >> spec.red_shift  );
      int g = ((c & spec.green_mask) >> spec.green_shift);
      int b = ((c & spec.blue_mask ) >> spec.blue_shift );
      int a = ((c & spec.alpha_mask) >> spec.alpha_shift);

      // If all alpha values = 0, we make the image opaque.
      if (!hasAlphaGreaterThanZero) {
        a = 255;

        // We cannot change the image spec (e.g. spec.alpha_mask=0) to
        // make the image opaque, because the "spec" of the image is
        // read-only. The image spec used by the client is the one
        // returned by get_image_spec().
      }
      // If there is alpha information and it's pre-multiplied alpha
      else if (hasValidPremultipliedAlpha) {
        if (a > 0) {
          // Convert it to straight alpha
          r = r * 255 / a;
          g = g * 255 / a;
          b = b * 255 / a;
        }
      }

      *dst =
        (r << spec.red_shift  ) |
        (g << spec.green_shift) |
        (b << spec.blue_shift ) |
        (a << spec.alpha_shift);
    }
  }
}

} // namespace details
} // namespace clip

#endif // CLIP_H_INCLUDED
