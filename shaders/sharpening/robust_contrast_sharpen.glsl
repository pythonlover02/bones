    c = clamp(c + (c * 4.0 - cross_sum) * 0.0625, min(box_min_x, c), max(box_max_x, c));
