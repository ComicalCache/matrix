#include <sys/ioctl.h>
#include <fcntl.h>

#include <sys/signal.h>

const uint64_t tiocgwinsz = TIOCGWINSZ;
const int32_t o_evtonly = O_EVTONLY;
const int32_t o_nonblock = O_NONBLOCK;

const int32_t sigint = SIGINT;
const int32_t sigterm = SIGTERM;
