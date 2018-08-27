#!/usr/bin/runhaskell
{-# Language OverloadedStrings #-}

import qualified Data.Text as Text
import qualified Data.Text.IO as IO

merge :: Text.Text -> Text.Text -> Text.Text
merge js template =
    Text.replace "{{js}}" js template

main :: IO ()
main = do
    js <- IO.readFile "output/index.js"
    template <- IO.readFile "index_template.html"
    IO.putStr $ merge js template

