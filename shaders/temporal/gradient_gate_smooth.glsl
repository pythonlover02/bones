    {
        float gx2 = dot(grad_x_rgb, grad_x_rgb);
        float gy2 = dot(grad_y_rgb, grad_y_rgb);
        float ge = clamp((gx2 + gy2) * 144.0, 0.0, 1.0);
        c = mix(c, mix(c, history, 0.6 * (1.0 - ge)), hist_valid);
    }
