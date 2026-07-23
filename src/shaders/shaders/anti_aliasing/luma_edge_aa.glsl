    {
        float lc = dot(c, LUMA_BT601);
        float ln = dot(tap_0_m1, LUMA_BT601);
        float ls = dot(tap_0_1, LUMA_BT601);
        float le = dot(tap_1_0, LUMA_BT601);
        float lw = dot(tap_m1_0, LUMA_BT601);
        float lnw = dot(tap_m1_m1, LUMA_BT601);
        float lne = dot(tap_1_m1, LUMA_BT601);
        float lsw = dot(tap_m1_1, LUMA_BT601);
        float lse = dot(tap_1_1, LUMA_BT601);
        float fmn = min(lc, min(min(ln, ls), min(le, lw)));
        float fmx = max(lc, max(max(ln, ls), max(le, lw)));
        vec2 fd = vec2(-((lnw + lne) - (lsw + lse)), (lnw + lsw) - (lne + lse));
        float fr = max((lnw + lne + lsw + lse) * 0.03125, 0.0078125);
        float fp = 1.0 / max(min(abs(fd.x), abs(fd.y)) + fr, 0.0001);
        fd = clamp(fd * fp, vec2(-8.0), vec2(8.0)) * inv;
        vec3 fa = (texture(u_input, v_uv + fd * -0.16666667).rgb +
                   texture(u_input, v_uv + fd *  0.16666667).rgb) * 0.5;
        vec3 fb = fa * 0.5 + (texture(u_input, v_uv + fd * -0.5).rgb +
                              texture(u_input, v_uv + fd *  0.5).rgb) * 0.25;
        float fl = dot(fb, LUMA_BT601);
        c = mix(fb, fa, clamp(step(fl, fmn) + step(fmx, fl), 0.0, 1.0));
    }
