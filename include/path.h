#ifndef XR0_PATH
#define XR0_PATH

#include <stdbool.h>

struct ast_function;
struct externals;
struct state;
struct error;

struct path;

struct path *
path_create(struct ast_function *, struct externals *);

void
path_destroy(struct path *);

bool
path_atend(struct path *);

struct error *
path_step(struct path *);

#endif
