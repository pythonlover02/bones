    {
        vec2 pg = u_resolution / max(4.0 * res_scale, 1.0);
        vec2 pt = v_uv * pg;
        vec2 pi = floor(pt - 0.5) + 0.5;
        vec2 pf = pt - pi;
        pf = pf * pf * (3.0 - 2.0 * pf);
        v_uv = (pi + pf) / pg;
    }
