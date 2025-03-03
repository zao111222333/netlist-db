import os
import pathlib
import polars

def obtain_datas(file:str | os.PathLike | pathlib.Path) -> dict[str, polars.DataFrame]: ...