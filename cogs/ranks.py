import typing
import discord
from discord.ext import commands

if typing.TYPE_CHECKING:
    from eikumi import Eikumi


class Ranks(commands.Cog):
    "The ranking system."

    def __init__(self, bot: commands.Bot):
        self.bot = bot
        self.eikumi: Eikumi = bot._eikumi  # type: ignore


async def setup(bot: commands.Bot) -> None:
    await bot.add_cog(Ranks(bot))
