# FCU

Utility application for F\*\* C\*\* that downloads missing images, bruteforces links, and so much more.

Note: currently this is solely a command-line application, and there are no plans on making a GUI for this.

## Save

Saves all of the images you have into a `all_images.txt` file

```sh
fcu save 

fcu         # this does the same as doing `fcu fill`
```

## Fill

Downloads all of the images you have a thumbnail, but don't have a full image of (and viceversa)

```sh
fcu fill
```

## Transfer

Transfers all of the full images and/or thumbnail images to a specific folder

```sh
fcu transfer -f "./images/full_images/" -t "./images/thumbnail_images"
```

## Bruteforce

Bruteforces all of the links by a given pattern

```sh
fcu bruteforce 0/\a\a # will bruteforce all `0/[a-z][a-z]` images (will give 3 images)
```

### Shortcuts

- `\_`
  - gets replaced with all combinations of  `[0-9a-z_]`
- `\d`
  - gets replaced with all combinations of `[0-9]`
- `\a`
  - gets replaced with all combinations of `[a-z]`
- `\m`
  - gets replaced with `0, 1, 2, 3, ... 98, 99, 100, 1000` (useful for the "mail" portion)
- `(abc)`
  - gets replaced with all combinations of `[abc]`, you can input any characters

## Stats

Will output some statistics about the images you have

```sh
fcu stats
```
