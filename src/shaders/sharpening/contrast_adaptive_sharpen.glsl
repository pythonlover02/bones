    {
        vec3 mn = min(box_min_3x3, c);
        vec3 mx = max(box_max_3x3, c);
        vec3 amp = sqrt(clamp(min(mn, 2.0 - mx) / max(mx, vec3(0.0001)), 0.0, 1.0));
        vec3 w = -(amp * 0.1625);
        c = clamp((cross_sum * w + c) / max(w * 4.0 + ONE3, vec3(0.0001)), 0.0, 1.0);
    }
