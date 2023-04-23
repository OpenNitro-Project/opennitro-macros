## Shim modifiers
A list of shim types compatible with the `#[bios_call]` attribute and `with_shim!` macro.

A lack of modifier produces the default behaviour - just routing through `BiosSafeShim`. Note that in this case, `with_shim!` is unnecessary at call sites.

### `expand64`
Used by the `Encrypt_64bit` and `Decrypt_64bit` functions to make the shim responsible for priming registers `r1` and `r2` with the split 64-bit word passed in `r1`.

### `expand64plus4`
Like `expand64`. Used only with `Encrypt_64bit`. `r1` passed to this shim has its value shifted down 4 bytes.

## Shim preload
Sometimes the shims need to set arbitrary values into the argument registers before the branch.
Usage example: `#[bios_call(arg3 = 0xC)]` creates a `mov r3, #0xc` at the top of the shim.

You can compose a modifier and a preload like so: `#[bios_call([expand64, arg3 = 0xC])]`.
_(Note the square brackets)_

## Generating multiple shims
You can comma-separate shims if you need a mix of different permutations of modifiers+preloads, like so: `#[bios_call([expand64, arg0 = 0x4], expand64plus4)]`.
However, calls to any shims with modifiers and/or preloads require the use of `with_shim!` at the call-site.

For example (at the call-site):
```
_ = with_shim!(0, MyAmazingShimmedFunc, args...);
```

The shim index maps to the shim order in the original `#[bios_call]` attribute.
Note that index 0 is **not** the default shim, but the first custom shim. For the default shim, omit `with_shim!`.

## Shims in the linker script

If for whatever reason, you're adding a new shimmed function, you will need to describe it in the linker script.
Their names will have the following format

Shim type       | Name prefix |
--------------- | ----------- |
No shim         | `RAW_`      |
Default shim    | None        |
`expand64`      | `EXP64_`    |
`expand64plus4` | `EXP64P4_`  |

Shims with preloads are expressed in the format given next. Their prefix is appended to the previous prefix. For example `EXP64_ARG04_FunctionName`.

`ARG<register_number><unprefixed_hex_value>_`

**WARNING**: There are clearly quite a few deficiencies in this approach (e.g. imagine the value being UINT32_MAX - ouch). This has been designed with the original BIOS in mind that doesn't run into these problems. You have been warned.
