import pandas as pd

df = pd.read_csv("dataset.csv")

df_filtered = df[["activity", "accel_x", "accel_y", "accel_z"]]

df_filtered.to_csv("preprocessed_dataset.csv", index=False)

print("Done. Removed evry column different from 'activity', 'accel_x', 'accel_y', and 'accel_z'.")