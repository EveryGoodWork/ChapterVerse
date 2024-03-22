import pandas as pd
import csv 

# Load the CSV file
file_names = ['bibles\\AMP.csv', 'bibles\\ASV.csv', 'bibles\\KJV.csv', 'bibles\\NKJV.csv', 'bibles\\WEB.csv']  # Add more file names as needed
# file_names = ['bibles\\AMP.csv']

# Define the lookup table for the 66 books of the Bible and their abbreviations
# Define the lookup table for book names and their abbreviations
books_lookup = {
    1: ("Genesis", "Gn"),
    2: ("Exodus", "Ex"),
    3: ("Leviticus", "Lv"),
    4: ("Numbers", "Nm"),
    5: ("Deuteronomy", "Dt"),
    6: ("Joshua", "Jos"),
    7: ("Judges", "Jdg"),
    8: ("Ruth", "Rt"),
    9: ("1 Samuel", "1Sm"),
    10: ("2 Samuel", "2Sm"),
    11: ("1 Kings", "1Kg"),
    12: ("2 Kings", "2Kg"),
    13: ("1 Chronicles", "1Ch"),
    14: ("2 Chronicles", "2Ch"),
    15: ("Ezra", "Ezr"),
    16: ("Nehemiah", "Neh"),
    17: ("Esther", "Est"),
    18: ("Job", "Jb"),
    19: ("Psalm", "Ps"),
    20: ("Proverbs", "Prv"),
    21: ("Ecclesiastes", "Ecc"),
    22: ("Song of Solomon", "SoS"),
    23: ("Isaiah", "Is"),
    24: ("Jeremiah", "Jer"),
    25: ("Lamentations", "Lm"),
    26: ("Ezekiel", "Ezk"),
    27: ("Daniel", "Dn"),
    28: ("Hosea", "Hos"),
    29: ("Joel", "Jl"),
    30: ("Amos", "Am"),
    31: ("Obadiah", "Ob"),
    32: ("Jonah", "Jon"),
    33: ("Micah", "Mic"),
    34: ("Nahum", "Nah"),
    35: ("Habakkuk", "Hab"),
    36: ("Zephaniah", "Zep"),
    37: ("Haggai", "Hag"),
    38: ("Zechariah", "Zec"),
    39: ("Malachi", "Mal"),
    40: ("Matthew", "Mt"),
    41: ("Mark", "Mk"),
    42: ("Luke", "Lk"),
    43: ("John", "Jn"),
    44: ("Acts", "Ac"),
    45: ("Romans", "Rm"),
    46: ("1 Corinthians", "1Co"),
    47: ("2 Corinthians", "2Co"),
    48: ("Galatians", "Gal"),
    49: ("Ephesians", "Eph"),
    50: ("Philippians", "Php"),
    51: ("Colossians", "Col"),
    52: ("1 Thessalonians", "1Th"),
    53: ("2 Thessalonians", "2Th"),
    54: ("1 Timothy", "1Tm"),
    55: ("2 Timothy", "2Tm"),
    56: ("Titus", "Ti"),
    57: ("Philemon", "Phm"),
    58: ("Hebrews", "Heb"),
    59: ("James", "Jas"),
    60: ("1 Peter", "1Pt"),
    61: ("2 Peter", "2Pt"),
    62: ("1 John", "1Jn"),
    63: ("2 John", "2Jn"),
    64: ("3 John", "3Jn"),
    65: ("Jude", "Jd"),
    66: ("Revelation", "Rev"),
}

for file_name in file_names:
    # Load the CSV file
    df = pd.read_csv(file_name)

    # Renaming 'id' column to 'reference' and 'book_id' to 'book'
    df.rename(columns={'id': 'reference', 'book_id': 'book'}, inplace=True)

    # Update the 'reference' field. Ensure you're using 'book', not 'book_id'
    df['reference'] = df.apply(lambda row: f"{books_lookup[row['book']][0]} {row['chapter']}:{row['verse']}", axis=1)
    
    # Insert the 'abbreviation' field right after the 'reference' field, using 'book'
    df.insert(
        loc=df.columns.get_loc('reference') + 1, 
        column='abbreviation', 
        value=df.apply(lambda row: f"{books_lookup[row['book']][1]}{row['chapter']}:{row['verse']}", axis=1)
    )

    # Export the DataFrame to a CSV file
    df.to_csv(file_name, index=False, quoting=csv.QUOTE_NONNUMERIC)

