<details>
<summary>XSD contract: <code>CodelistType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="CodelistType">
		<xs:complexContent>
			<xs:extension base="CodelistBaseType">
				<xs:sequence>
					<xs:element name="CodelistExtension" type="CodelistExtensionType" minOccurs="0" maxOccurs="unbounded">
						<xs:annotation>
							<xs:documentation>CodelistExtension allows for the extension of codelists by referencing the codelists to be extended and providing inclusion/exclusion rules for selecting the codes to extend. The order of these is important as it is indicates the order of precedence of the extended codelists for conflict resolution of codes. However, the prefix property can be used to ensure uniqueness of inherited codes in the extending codelist, in case conflicting codes must be included.</xs:documentation>
						</xs:annotation>
					</xs:element>
				</xs:sequence>
			</xs:extension>
		</xs:complexContent>
	</xs:complexType>
```

</details>
