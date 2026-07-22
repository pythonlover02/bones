    {
        vec3 ab = cross_sum * 0.2 + corner_sum * 0.05;
        vec3 ad = c - ab;
        float ae = abs(ad.r) + abs(ad.g) + abs(ad.b);
        c = c + ad * (0.6 / max(1.0 + ae * 4.0, 0.0001));
    }
