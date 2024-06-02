def future_value_of_series(P, r_percent, years):
    r = r_percent / 100  # Convert percentage to a decimal
    n = years * 12  # Convert years to months

    FV = P * (((1 + r) ** n - 1) / r)
    return FV