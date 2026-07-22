    {
        float hmx = max(c.r, max(c.g, c.b));
        float hmn = min(c.r, min(c.g, c.b));
        float hd = hmx - hmn;
        float hl = (hmx + hmn) * 0.5;
        vec3 d3 = vec3(hmx) - c;
        float invd = 1.0 / max(hd, 1e-5);
        vec3 hcand = vec3(
            (d3.g - d3.b) * invd,
            (d3.b - d3.r) * invd + 2.0,
            (d3.r - d3.g) * invd + 4.0
        );
        float pick_r = step(hmx - 1e-5, c.r);
        float pick_g = step(hmx - 1e-5, c.g) * (1.0 - pick_r);
        float pick_b = (1.0 - pick_r) * (1.0 - pick_g);
        float hue_raw = hcand.x * pick_r + hcand.y * pick_g + hcand.z * pick_b;
        float hue = fract(hue_raw * 0.16666667) * step(1e-5, hd);
        float hs = clamp(hd / max(1.0 - abs(2.0 * hl - 1.0), 0.0001) * 1.1, 0.0, 1.0);
        float hl2 = clamp(hl, 0.0, 1.0);
        float hc = (1.0 - abs(2.0 * hl2 - 1.0)) * hs;
        float hm = hl2 - hc * 0.5;
        vec3 hbase = clamp(abs(mod(hue * 6.0 + vec3(0.0, 4.0, 2.0), 6.0) - 3.0) - 1.0, 0.0, 1.0);
        c = hbase * hc + vec3(hm);
    }
