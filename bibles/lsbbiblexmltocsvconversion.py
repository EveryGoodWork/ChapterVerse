import os
import pandas as pd
import xml.etree.ElementTree as ET

# Set up base directory (the directory where the script is executed from)
base_directory = os.getcwd()  # This gets the current working directory

# Relative paths for the input and output, ensure 'bibles' is included only once
input_path = os.path.join(base_directory, 'bibles', 'LSB.xml')
output_path = os.path.join(base_directory, 'bibles', 'LSB.csv')

# Ensure that paths point correctly by printing them (you can remove these print statements later)
print("Input path:", input_path)
print("Output path:", output_path)

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

# Tags to be removed
tags_to_remove = ['<BN>', '<CN>', '<SH>', '<SB>', '<SN>', "<SF>", "<SS>"]


def remove_selected_lines(input_path, output_path, tags):
    # Read the input file
    with open(input_path, 'r', encoding='utf-8') as file:
        lines = file.readlines()

    # Filter lines that do not contain the specified tags
    filtered_lines = [line for line in lines if not any(
        tag in line for tag in tags)]

    # Write the filtered lines to the output file
    with open(output_path, 'w', encoding='utf-8') as file:
        file.writelines(filtered_lines)


def modify_and_clean_tags(input_path, output_path):
    # Read the input file
    with open(input_path, 'r', encoding='utf-8') as file:
        lines = file.readlines()

    # Modify and clean the tags in the lines
    modified_lines = []
    for line in lines:

        # Step 1: Remove complex patterns involving quotes and tags
        # Removes the "<RS>“" opening tag and quote
        # line = line.replace('<RS>“', '')
        # Removes the closing quote and "</RS>" tag
        # line = line.replace('”</RS>', '')
        # Removes any standalone "<RS>" tags

        line = line.replace('{{', '')
        line = line.replace('}}', ',')

        line = line.replace('<RS>', '')
        line = line.replace('<RS>+', '')
        line = line.replace('</RS>', '')
        line = line.replace('+', '')

        replacements = ['<RS>', '<RS>+', '</RS>', '+', '<C>', '<A>', '<PM>', '<V>', '<P>', '<CP>', '<CC>', '<\\>', '</>',
                        '<PO>', '<PN>-', '<B>', '</B>', '<HL>', '<HLL>', '<LL>', '<LLL>', '<PN>', '<PR>', '<SHI>', '</SHI>',
                        '<BR>', '</BR>']

        for rep in replacements:
            line = line.replace(rep, '')

        # Replace <T> with a comma and wrap the following text in quotes
        if '<T>' in line:
            parts = line.split('<T>')
            line = parts[0] + ', "' + parts[1].strip() + '"\n'
        modified_lines.append(line)

    # Write the modified lines to the output file
    with open(output_path, 'w', encoding='utf-8') as file:
        file.writelines(modified_lines)


def format_scripture_references(input_path, output_path, books_lookup):
    with open(input_path, 'r', encoding='utf-8') as file:
        lines = file.readlines()

    # Start with the headers
    formatted_lines = [
        '"reference","abbreviation","book","chapter","verse","scripture"\n']
    for line in lines:
        parts = line.strip().split('::')
        if len(parts) < 2:
            continue  # Skip lines that do not follow the "number::" pattern

        book_num = int(parts[0])
        # Split into three parts allowing the scripture to contain commas
        chapter_verse_scripture = parts[1].split(',', 2)

        if len(chapter_verse_scripture) < 3:
            continue  # Skip lines that do not have a chapter, verse, and scripture part

        chapter, verse, scripture = chapter_verse_scripture

        # Handle scripture text with quotes correctly
        scripture = scripture.strip()
        if scripture.startswith('"') and scripture.endswith('"'):
            scripture = scripture  # Assuming it's already properly quoted
        else:
            scripture = f'"{scripture}"'  # Ensure scripture is quoted

        if book_num in books_lookup:
            book_name, book_abbr = books_lookup[book_num]
            reference = f'"{book_name} {chapter}:{verse}"'
            abbreviation = f'"{book_abbr}{chapter}:{verse}"'
            formatted_line = f'{reference},{abbreviation},{book_num},{chapter},{verse},{scripture}\n'
            formatted_lines.append(formatted_line)

    with open(output_path, 'w', encoding='utf-8') as file:
        file.writelines(formatted_lines)


# Call the function with the specified paths and tags
remove_selected_lines(input_path, output_path, tags_to_remove)
modify_and_clean_tags(output_path, output_path)
format_scripture_references(output_path, output_path, books_lookup)
