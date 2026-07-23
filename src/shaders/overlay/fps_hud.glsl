    {
        vec2 hp = hud_raw;
        hp.x += (1.0 - hp.y) * 0.15;
        float fps_clamped = clamp(u_fps, 0.0, 9999.0);
        float fps_digit3 = mod(floor(fps_clamped * 0.001), 10.0);
        float fps_digit2 = mod(floor(fps_clamped * 0.01), 10.0);
        float fps_digit1 = mod(floor(fps_clamped * 0.1), 10.0);
        float fps_digit0 = mod(floor(fps_clamped), 10.0);
        float show3 = step(1.0, fps_digit3);
        float show2 = step(1.0, fps_digit3 + fps_digit2);
        float show1 = step(1.0, fps_digit3 + fps_digit2 + fps_digit1);
        float d3 = mix(999.0, hud_digit(hp, fps_digit3), show3);
        float d2 = mix(999.0, hud_digit(hp - vec2(0.7, 0.0), fps_digit2), show2);
        float d1 = mix(999.0, hud_digit(hp - vec2(1.4, 0.0), fps_digit1), show1);
        float d0 = hud_digit(hp - vec2(2.1, 0.0), fps_digit0);
        float d_text = min(min(d3, d2), min(d1, d0));
        float text_core = (1.0 - smoothstep(0.045, 0.075, d_text)) * hud_box;
        float text_glow = (1.0 - smoothstep(0.06, 0.35, d_text)) * hud_box;
        vec2 bg_d = abs(hud_raw - vec2(1.55, 0.45)) - vec2(1.4, 0.2);
        float bg_dist = length(max(bg_d, 0.0)) + min(max(bg_d.x, bg_d.y), 0.0);
        float bg_alpha = (1.0 - smoothstep(0.1, 0.5, bg_dist)) * 0.75 * hud_box;
        c = mix(c, vec3(0.01, 0.02, 0.05), bg_alpha);
        c = mix(c, vec3(0.0, 0.5, 1.0), text_glow * 0.85);
        c = mix(c, vec3(0.85, 0.95, 1.0), text_core);
    }
