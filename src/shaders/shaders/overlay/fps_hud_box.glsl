    float hud_hs = 26.0 * res_scale;
    vec2 hud_raw = (v_uv - vec2(0.012)) * u_resolution / hud_hs;
    float hud_box = step(-0.6, hud_raw.x) * step(hud_raw.x, 3.5)
                  * step(-0.6, hud_raw.y) * step(hud_raw.y, 1.6);
