QBasic is fun. And we can syntax highlight the below code here via loading custom syntax definitions from a path
specified in the `server.yml` file. Unfortunately, this only supports loading from `.sublime-text` files, **not**
from `.tmLanguage` files.

```bas
DEFINT A-Z
SCREEN 13
DEF SEG = &HA000

CLS
DO UNTIL INKEY$ = CHR$(27)
    x = RND * 320
    y = RND * 200
    clr = RND * 256
    POKE (y * 320) + x, clr
LOOP
```