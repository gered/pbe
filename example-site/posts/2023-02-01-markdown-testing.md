Here we test some Markdown (actually, [CommonMark](https://commonmark.org/)) things, gloriously rendered via the
[pulldown-cmark](https://github.com/raphlinus/pulldown-cmark) crate!

---

This is a sentence rendered as a paragraph.

This is a sentence that is also rendered as a paragraph,  
but it has a forced line-break in the middle of it!

This is normal text.

_This is italicized text._

*This is also italicized text.*

**This is bolded text.**

__This is also bolded text.__

~~This is strikethrough text.~~

We can also **escape** characters to skip applying formatting like \_so_!

# Heading Level 1
## Heading Level 2
### Heading Level 3
#### Heading Level 4
##### Heading Level 5
###### Heading Level 6

Heading Level 1 Alternate
=========================

Heading Level 2 Alternate
-------------------------

> This is rendered as a block quote.

Also ...

> This is rendered as a multi-line block quote.
> This is in the same block quote.
> 
> And finally, this is also in the same block quote!

But wait, there's more!

> Block quote again.
> > Nested block quote action! Wow!

1. Number one
2. Number two
3. Number three

- First
- Second
- And finally, third!

1. One item which is multi-line.
   This is the second line of the first item.
2. Here is item number two.
3. And item number three.

* First item
    * First item, first sub-item
    * First item, second sub-item
* Second item
* Third item

1. First item
   1. First item, first sub-item
   2. First item, second sub-item
2. Second item
3. Third item

* More lists
* Because why not?
* It's getting a little tiresome ...
* So this will be the last one!

Escaping to prevent accidental un-ordered list rendering ...

\* Like so 

Task lists!

- [ ] Task 1
- [x] Task 2 (completed!)

Here's some tables. To be honest, this syntax really sucks for anything but the very simplest of tables ...

| foo | bar | longer column heading |
|-----|-----|-----------------------|
| yes | no  | hi                    |
| 1   | 2   | 3                     |

| left | right |
|:-----|------:|
| a    | b     |
| one  | two   |

| lazy | cell | formatting |
|------|------|------------|
| a | b | c |
| one | two | three |

Here's a code block.

```
#include <stdio.h>

int main(int argc, char *argv[]) {
    printf("Hello, world!\n");
    return 0;
}
```

Here's a code block with a language specified (which could be syntax highlighted if the right server-side parsing
was happening ...).

```c
#include <stdio.h>

int main(int argc, char *argv[]) {
    printf("Hello, world!\n");
    return 0;
}
```

Code blocks can also be indented like so:

    #include <stdio.h>
    
    int main(int argc, char *argv[]) {
        printf("Hello, world!\n");
        return 0;
    }


Here's some inline `code` bits that will appear `inline` within this paragraph.

<div class="foobar">
We can also render <strong>html</strong> inline.
</div>

And we can place horizontal rules:

---

And we can have images!

![An image](/images/coffee_and_donuts.jpg)

Images can also be links:

[![An image](/images/coffee_and_donuts.jpg)](/images/coffee_and_donuts.jpg)

