#!/bin/sh

if [ -z "${OPENSCAD}" ]
then
    OPENSCAD=`which openscad`
fi

if [ -z "${OPENSCAD}" ]
then
    >&2 echo -e "Unable to find openscad. Add to your path or set the variable 'OPENSCAD'"
fi

MODEL_PATH=$(dirname $(realpath "${0}"))/../models
for scad_file in `find "${MODEL_PATH}" -iname "*.scad"`
do
    outname=$(dirname ${scad_file})/$(basename $scad_file .scad)
    $OPENSCAD --export-format binstl -o ${outname}-bin.stl $scad_file
    $OPENSCAD --export-format asciistl -o ${outname}-ascii.stl $scad_file
done
