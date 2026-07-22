    {
        float ll = dot(c, LUMA_BT601);
        float la = dot(cross_avg, LUMA_BT601);
        float ld = clamp(ll - la, -0.1, 0.1);
        c = c + vec3(ld);
    }
