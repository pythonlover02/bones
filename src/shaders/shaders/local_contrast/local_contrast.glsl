    {
        float ll = dot(c, LUMA_AVG);
        float la = dot(cross_avg, LUMA_AVG);
        c = c * (1.0 + (ll - la) * 0.3 / max(ll, 0.0001));
    }
