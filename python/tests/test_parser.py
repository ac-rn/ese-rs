"""
Tests for the ESE parser Python bindings.
"""

import os
import tempfile
import pytest
from pathlib import Path

# Import from the built module
try:
    from ese_parser import EseDatabase, __version__
except ImportError:
    pytest.skip("ese_parser module not built", allow_module_level=True)


# Path to test databases (adjust as needed)
TEST_DB_DIR = Path(__file__).parent.parent.parent
CURRENT_MDB = TEST_DB_DIR / "Current.mdb"
SRUDB_DAT = TEST_DB_DIR / "SRUDB.dat"


class TestEseDatabase:
    """Test the EseDatabase class."""

    def test_version(self):
        """Test that version is accessible."""
        assert __version__ is not None
        assert isinstance(__version__, str)

    def test_open_database(self):
        """Test opening a database."""
        if not CURRENT_MDB.exists():
            pytest.skip(f"Test database not found: {CURRENT_MDB}")
        
        db = EseDatabase(str(CURRENT_MDB))
        assert db is not None
        assert db.path == str(CURRENT_MDB)

    def test_open_nonexistent_database(self):
        """Test opening a non-existent database raises error."""
        with pytest.raises(FileNotFoundError):
            EseDatabase("nonexistent_database.mdb")

    def test_database_properties(self):
        """Test database properties."""
        if not CURRENT_MDB.exists():
            pytest.skip(f"Test database not found: {CURRENT_MDB}")
        
        db = EseDatabase(str(CURRENT_MDB))
        
        # Check page size
        assert db.page_size > 0
        assert db.page_size in [4096, 8192, 16384, 32768]
        
        # Check total pages
        assert db.total_pages > 0

    def test_get_tables(self):
        """Test getting list of tables."""
        if not CURRENT_MDB.exists():
            pytest.skip(f"Test database not found: {CURRENT_MDB}")
        
        db = EseDatabase(str(CURRENT_MDB))
        tables = db.get_tables()
        
        assert isinstance(tables, list)
        assert len(tables) > 0
        
        # MSysObjects should exist in all ESE databases
        assert "MSysObjects" in tables

    def test_read_table(self):
        """Test reading a table."""
        if not CURRENT_MDB.exists():
            pytest.skip(f"Test database not found: {CURRENT_MDB}")
        
        db = EseDatabase(str(CURRENT_MDB))
        records = db.read_table("MSysObjects")
        
        assert isinstance(records, list)
        assert len(records) > 0
        
        # Check that records are dictionaries
        assert isinstance(records[0], dict)
        
        # MSysObjects should have these columns
        assert "Name" in records[0] or "ObjidTable" in records[0]

    def test_read_nonexistent_table(self):
        """Test reading a non-existent table raises error."""
        if not CURRENT_MDB.exists():
            pytest.skip(f"Test database not found: {CURRENT_MDB}")
        
        db = EseDatabase(str(CURRENT_MDB))
        
        with pytest.raises(ValueError, match="Table not found"):
            db.read_table("NonExistentTable123456")

    def test_get_table_schema(self):
        """Test getting table schema."""
        if not CURRENT_MDB.exists():
            pytest.skip(f"Test database not found: {CURRENT_MDB}")
        
        db = EseDatabase(str(CURRENT_MDB))
        schema = db.get_table_schema("MSysObjects")
        
        assert isinstance(schema, list)
        assert len(schema) > 0
        
        # Check column info structure
        col = schema[0]
        assert "name" in col
        assert "type" in col
        assert "id" in col
        assert "nullable" in col

    def test_export_table(self):
        """Test exporting a table to JSONL."""
        if not CURRENT_MDB.exists():
            pytest.skip(f"Test database not found: {CURRENT_MDB}")
        
        db = EseDatabase(str(CURRENT_MDB))
        
        with tempfile.TemporaryDirectory() as tmpdir:
            output_file = Path(tmpdir) / "MSysObjects.jsonl"
            db.export_table("MSysObjects", str(output_file))
            
            # Check file was created
            assert output_file.exists()
            
            # Check file has content
            assert output_file.stat().st_size > 0
            
            # Check it's valid JSONL
            import json
            with open(output_file) as f:
                lines = f.readlines()
                assert len(lines) > 0
                
                # Parse first line as JSON
                record = json.loads(lines[0])
                assert isinstance(record, dict)

    def test_export_all(self):
        """Test exporting all tables."""
        if not CURRENT_MDB.exists():
            pytest.skip(f"Test database not found: {CURRENT_MDB}")
        
        db = EseDatabase(str(CURRENT_MDB))
        
        with tempfile.TemporaryDirectory() as tmpdir:
            db.export_all(tmpdir)
            
            # Check that files were created
            output_files = list(Path(tmpdir).glob("*.jsonl"))
            assert len(output_files) > 0
            
            # Check MSysObjects was exported
            msy_objects_file = Path(tmpdir) / "MSysObjects.jsonl"
            assert msy_objects_file.exists()

    def test_context_manager(self):
        """Test using database as context manager."""
        if not CURRENT_MDB.exists():
            pytest.skip(f"Test database not found: {CURRENT_MDB}")
        
        with EseDatabase(str(CURRENT_MDB)) as db:
            tables = db.get_tables()
            assert len(tables) > 0

    def test_repr(self):
        """Test string representation."""
        if not CURRENT_MDB.exists():
            pytest.skip(f"Test database not found: {CURRENT_MDB}")
        
        db = EseDatabase(str(CURRENT_MDB))
        repr_str = repr(db)
        
        assert "EseDatabase" in repr_str
        assert "tables=" in repr_str
        assert "page_size=" in repr_str


class TestSRUDB:
    """Test with SRUDB.dat if available."""

    def test_srudb_tables(self):
        """Test reading SRUDB.dat tables."""
        if not SRUDB_DAT.exists():
            pytest.skip(f"SRUDB.dat not found: {SRUDB_DAT}")
        
        db = EseDatabase(str(SRUDB_DAT))
        tables = db.get_tables()
        
        # SRUDB should have these tables
        expected_tables = ["SruDbIdMapTable", "MSysObjects"]
        for table in expected_tables:
            assert table in tables, f"Expected table {table} not found"

    def test_srudb_read_table(self):
        """Test reading a table from SRUDB.dat."""
        if not SRUDB_DAT.exists():
            pytest.skip(f"SRUDB.dat not found: {SRUDB_DAT}")
        
        db = EseDatabase(str(SRUDB_DAT))
        
        # Try to read SruDbIdMapTable
        if "SruDbIdMapTable" in db.get_tables():
            records = db.read_table("SruDbIdMapTable")
            assert isinstance(records, list)
            # May be empty, but should be a list
            assert len(records) >= 0


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
