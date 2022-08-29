/* xdo utility pieces 
 *
 * $Id$
 */

#ifndef _XDO_UTIL_H_
#define _XDO_UTIL_H_

#include "xdo.h"

/* human to Keysym string mapping */
static const char *symbol_map[] = {
  "alt", "Alt_L",
  "ctrl", "Control_L",
  "control", "Control_L",
  "meta", "Meta_L",
  "super", "Super_L",
  "shift", "Shift_L",
  "enter", "Return",
  "return", "Return",
  NULL, NULL,
};

#endif /* ifndef _XDO_UTIL_H_ */
