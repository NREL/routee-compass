# TODO: this information should be stored with the actual models in the form of metadata
MODELS_POWERTRAINS = {
    "2012_Ford_Focus": "ICE",
    "2012_Ford_Fusion": "ICE",
    "2016_AUDI_A3_4cyl_2WD": "ICE",
    "2016_BMW_328d_4cyl_2WD": "ICE",
    "2016_CHEVROLET_Malibu_4cyl_2WD": "ICE",
    "2016_CHEVROLET_Spark_EV": "BEV",
    "2016_FORD_C-MAX_HEV": "HEV",
    "2016_FORD_Escape_4cyl_2WD": "ICE",
    "2016_FORD_Explorer_4cyl_2WD": "ICE",
    "2016_HYUNDAI_Elantra_4cyl_2WD": "ICE",
    "2016_KIA_Optima_Hybrid": "HEV",
    "2016_Leaf_24_kWh": "BEV",
    "2016_MITSUBISHI_i-MiEV": "BEV",
    "2016_Nissan_Leaf_30_kWh": "BEV",
    "2016_TESLA_Model_S60_2WD": "BEV",
    "2016_TOYOTA_Camry_4cyl_2WD": "ICE",
    "2016_TOYOTA_Corolla_4cyl_2WD": "ICE",
    "2016_TOYOTA_Highlander_Hybrid": "HEV",
    "2016_Toyota_Prius_Two_FWD": "HEV",
    "2017_CHEVROLET_Bolt": "BEV",
    "2017_Maruti_Dzire_VDI": "ICE",
    "2017_Toyota_Highlander_3.5_L": "ICE",
    "2020_Chevrolet_Colorado_2WD_Diesel": "ICE",
    "2020_VW_Golf_1.5TSI": "ICE",
    "2020_VW_Golf_2.0TDI": "ICE",
    "2021_Fiat_Panda_Mild_Hybrid": "ICE",
    "2021_Peugot_3008": "ICE",
    "2022_Ford_F-150_Lightning_4WD": "BEV",
    "2022_Renault_Zoe_ZE50_R135": "BEV",
    "2022_Tesla_Model_3_RWD": "BEV",
    "2022_Tesla_Model_Y_RWD": "BEV",
    "2022_Toyota_Yaris_Hybrid_Mid": "HEV",
    "2022_Volvo_XC40_Recharge_twin": "BEV",
    "2023_Mitsubishi_Pajero_Sport": "ICE",
}

ADJUSTMENT_FACTORS = {
    "ICE": 1.166,
    "HEV": 1.1252,
    "BEV": 1.3958,
}

ENERGY_OUTPUT_UNITS = {
    "ICE": "gallons_gasoline_per_mile",
    "HEV": "gallons_gasoline_per_mile",
    "BEV": "kilowatt_hours_per_mile",
}

IDEAL_ENERGY_RATES = {
    "ICE": 1 / 35,  # 35 mpg
    "HEV": 1 / 50,  # 50 mpg
    "BEV": 0.2,  # 20 kWh per 100 miles
}
