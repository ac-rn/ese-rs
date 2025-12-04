"""Type stubs for ese_parser"""

from typing import Any, Dict, List, Optional
from types import TracebackType

__version__: str

class EseDatabase:
    """
    ESE Database parser.
    
    Provides access to Microsoft ESE databases with high performance.
    """
    
    def __init__(self, path: str) -> None:
        """
        Open an ESE database.
        
        Args:
            path: Path to the database file (.mdb, .edb, .dat)
            
        Raises:
            FileNotFoundError: If the database file doesn't exist
            IOError: If the database cannot be opened or is invalid
        """
        ...
    
    @property
    def path(self) -> str:
        """Get the path to the database file."""
        ...
    
    @property
    def page_size(self) -> int:
        """Get the database page size in bytes."""
        ...
    
    @property
    def total_pages(self) -> int:
        """Get the total number of pages in the database."""
        ...
    
    def get_tables(self) -> List[str]:
        """
        Get a list of all table names in the database.
        
        Returns:
            List of table names
        """
        ...
    
    def read_table(self, table_name: str) -> List[Dict[str, Any]]:
        """
        Read all records from a table.
        
        Args:
            table_name: Name of the table to read
            
        Returns:
            List of records as dictionaries
            
        Raises:
            ValueError: If the table doesn't exist
            IOError: If there's an error reading the table
        """
        ...
    
    def get_table_schema(self, table_name: str) -> List[Dict[str, Any]]:
        """
        Get the schema (columns) for a table.
        
        Args:
            table_name: Name of the table
            
        Returns:
            List of column information dictionaries with keys:
            - name: Column name
            - type: Column type
            - id: Column identifier
            - nullable: Whether the column can be null
            
        Raises:
            ValueError: If the table doesn't exist
        """
        ...
    
    def export_table(self, table_name: str, output_path: str) -> None:
        """
        Export a table to JSONL format.
        
        Args:
            table_name: Name of the table to export
            output_path: Path to the output JSONL file
            
        Raises:
            ValueError: If the table doesn't exist
            IOError: If there's an error writing the file
        """
        ...
    
    def export_all(self, output_dir: str) -> None:
        """
        Export all tables to JSONL files in a directory.
        
        Args:
            output_dir: Directory to write JSONL files to
            
        Raises:
            IOError: If there's an error creating the directory or writing files
        """
        ...
    
    def __enter__(self) -> "EseDatabase":
        """Context manager entry."""
        ...
    
    def __exit__(
        self,
        exc_type: Optional[type],
        exc_value: Optional[BaseException],
        traceback: Optional[TracebackType],
    ) -> bool:
        """Context manager exit."""
        ...
    
    def __repr__(self) -> str:
        """String representation."""
        ...

__all__ = ["EseDatabase", "__version__"]
