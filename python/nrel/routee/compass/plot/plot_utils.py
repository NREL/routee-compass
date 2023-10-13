class ColormapCircularIterator:
    def __init__(self, colormap, num_colors):
        self.colormap = colormap
        self.num_colors = num_colors
        self.index = 0

    def __iter__(self):
        return self

    def __next__(self):
        if not self.colormap:
            raise StopIteration
        normalized_value = self.index / float(self.num_colors)
        rgba = self.colormap(normalized_value)
        hex_color = rgba_to_hex(rgba)
        self.index = (self.index + 1) % self.num_colors
        return hex_color


def rgba_to_hex(rgba):
    return "#{:02x}{:02x}{:02x}".format(
        int(rgba[0] * 255), int(rgba[1] * 255), int(rgba[2] * 255)
    )
