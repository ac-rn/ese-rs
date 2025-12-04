"""
ESE Parser - Fast ESE database parser with Rust backend

This package provides a high-performance Python interface to Microsoft ESE
(Extensible Storage Engine) databases, commonly used in Windows systems.

Example:
    >>> from ese_parser import EseDatabase
    >>> 
    >>> # Open a database
    >>> db = EseDatabase("Current.mdb")
    >>> 
    >>> # List tables
    >>> tables = db.get_tables()
    >>> print(f"Found {len(tables)} tables")
    >>> 
    >>> # Read a table
    >>> records = db.read_table("MSysObjects")
    >>> for record in records:
    ...     print(record)
    >>> 
    >>> # Export to JSONL
    >>> db.export_table("MSysObjects", "output.jsonl")
    >>> 
    >>> # Export all tables
    >>> db.export_all("output_dir/")
    >>> 
    >>> # Context manager support
    >>> with EseDatabase("Current.mdb") as db:
    ...     records = db.read_table("MSysObjects")
"""

from .ese_parser import EseDatabase, __version__

__all__ = ["EseDatabase", "__version__"]
