    {
        const vec3 THR_GHOST = vec3(0.7);
        vec2 gc = HALF2 - v_uv;
        vec2 gc_step = gc * 0.2;
        vec3 gg = max(texture(u_input, v_uv + gc_step).rgb - THR_GHOST, ZERO3)
                + max(texture(u_input, v_uv + gc_step * 2.0).rgb - THR_GHOST, ZERO3)
                + max(texture(u_input, v_uv + gc_step * 3.0).rgb - THR_GHOST, ZERO3);
        c = c + gg * 0.4 * (1.0 - clamp(dot(gc, gc), 0.0, 1.0));
    }
