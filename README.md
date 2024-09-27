# Sigma File Extension (SFE)

The Sigma File Extension is a alternative to PNG. It offeres a unique and open source tool for creating and converting images from and to PNG.

## TODO

1. Work on a more user friendly method to convert images
2. Work on making it possible to view SFE files
3. Convert to tauri later down the road when it's almost complete
4. Add a GitHub workflow to compile and release SFE automatically or when needed

## Known Issues

1. Takes a long time on lower end PCs/laptops to convert a image
2. Images only write to 256x256 even on a larger canvas when converting from sigma to png
3. [Unconfirmed] Image previewer will not restrict the scale of images
4. [Intentional, but don't know any other solution] Image transparency is rendered as white in Image previewer
5. Debug window stays open when previewing an image, don't know how to fix it
