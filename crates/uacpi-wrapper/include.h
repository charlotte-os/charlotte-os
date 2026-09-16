/* Translation unit bindgen parses to produce uACPI's public surface. It
 * deliberately omits everything under uacpi/internal/: those headers are the
 * interpreter's own business and pull in layouts that callers are not meant to
 * depend on. uacpi/platform/ is reached transitively, since the public headers
 * are written in terms of the uacpi_* scalar typedefs it declares. */

#include <uacpi/uacpi.h>

#include <uacpi/acpi.h>
#include <uacpi/context.h>
#include <uacpi/event.h>
#include <uacpi/io.h>
#include <uacpi/kernel_api.h>
#include <uacpi/log.h>
#include <uacpi/namespace.h>
#include <uacpi/notify.h>
#include <uacpi/opregion.h>
#include <uacpi/osi.h>
#include <uacpi/registers.h>
#include <uacpi/resources.h>
#include <uacpi/sleep.h>
#include <uacpi/status.h>
#include <uacpi/tables.h>
#include <uacpi/types.h>
#include <uacpi/utilities.h>
