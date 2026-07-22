    {
        const float INK_EDGE_LO = 0.003;
        const float INK_EDGE_HI = 0.025;
        const float INK_STRENGTH = 0.85;
        const vec3 INK_COLOR = vec3(0.03, 0.03, 0.05);
        float ink_g2 = dot(grad_x_rgb, grad_x_rgb) + dot(grad_y_rgb, grad_y_rgb);
        c = mix(c, INK_COLOR, smoothstep(INK_EDGE_LO, INK_EDGE_HI, ink_g2) * INK_STRENGTH);
    }
