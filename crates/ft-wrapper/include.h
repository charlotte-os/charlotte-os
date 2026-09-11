/* Translation unit bindgen parses to produce Flanterm's public surface. It
 * deliberately omits the *_private.h headers: without FLANTERM_IN_FLANTERM
 * defined, `struct flanterm_context` stays opaque, which is exactly how callers
 * are meant to see it. */

#include "flanterm.h"
#include "flanterm_backends/fb.h"
